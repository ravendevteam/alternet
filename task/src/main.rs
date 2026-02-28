use clap::Parser as _;
use std::io::Write as _;
use futures_util::TryStreamExt as _;
use std::io::Read as _;
use std::io::Write as _;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(clap::Parser)]
struct Main {
    #[command(subcommand)]
    command: Command
}

#[derive(clap::Subcommand)]
enum Command {
    BuildImage,
    
    #[command(name = "build-node")]
    BuildNode,

    #[command(name = "build-node-release")]
    BuildNodeRelease
}

pub fn inject_bin_and_archive(
    crate_dir: &std::path::Path, 
    injection_bin_path: &std::path::Path,
    injection_bin_name: &str
) -> Result<Vec<u8>> {
    let mut tar: tar::Builder<_> = tar::Builder::new(vec![]);
    for item in walkdir::WalkDir::new(crate_dir) {
        let item: walkdir::DirEntry = item?;
        let item_path: &std::path::Path = item.path();
        let relative_path: &std::path::Path = item_path.strip_prefix(crate_dir)?;
        if item_path == crate_dir {
            continue
        }
        if item_path.is_file() {
            let mut file: std::fs::File = std::fs::File::open(&item_path)?;
            tar.append_file(relative_path, &mut file)?;
        } else if item_path.is_dir() {
            tar.append_dir(relative_path, &item_path)?;
        }
    }
    let mut bin_file = std::fs::File::open(injection_bin_path)?;
    tar.append_file(injection_bin_name, &mut bin_file)?;
    tar.finish()?;
    let bytes: Vec<u8> = tar.into_inner()?;
    Ok(bytes)
}

fn bin_to_crate() -> Result<Vec<(String, std::path::PathBuf)>> {
    let output: std::process::Output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .output()?;
    if !output.status.success() {
        let error: Box<dyn std::error::Error> = "cargo metadata failed".into();
        return Err(error)
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let packages: &Vec<_> = metadata["packages"].as_array().ok_or("no packages")?;
    let mut bins: Vec<_> = vec![];
    for package in packages {
        let manifest_path: &str = package["manifest_path"].as_str().ok_or("missing manifest_path")?;
        let crate_dir: std::path::PathBuf = manifest_path.into();
        let crate_dir: std::path::PathBuf = crate_dir
            .parent()
            .ok_or("manifest_path has no parent")?
            .to_path_buf();
        let targets: &Vec<_> = package["targets"].as_array().ok_or("missing targets")?;
        for target in targets {
            let kind: &Vec<_> = target["kind"].as_array().ok_or("missing kind")?;
            if kind.iter().any(|k| k.as_str() == Some("bin")) {
                let name: String = target["name"]
                    .as_str()
                    .ok_or("missing target name")?
                    .to_string();
                bins.push((name, crate_dir.clone()));
            }
        }
    }
    Ok(bins)
}




trait DockerExt {
    async fn snapshot(
        &self, 
        ws_root: &std::path::Path, 
        ws_root_dockerfile_rel_path: &std::path::Path,
        ws_root_exclude: &[std::path::PathBuf],
        image_name: &str,
        image_tag: &str,
        image_out_dir: &std::path::Path
    ) -> Result<()>;
}

impl DockerExt for bollard::Docker {
    async fn snapshot(
        &self, 
        ws_root: &std::path::Path, 
        ws_root_dockerfile_rel_path: &std::path::Path,
        ws_root_exclude: &[std::path::PathBuf],
        image_name: &str,
        image_tag: &str,
        image_out_dir: &std::path::Path
    ) -> Result<()> {
        let mut tar: tar::Builder<_> = tar::Builder::new(vec![]);
        let walker = walkdir::WalkDir::new(ws_root)
            .into_iter()
            .filter_entry(|item| {
                let rel = item
                    .path()
                    .strip_prefix(ws_root)
                    .unwrap_or(item.path());
                let rel_str = rel.to_string_lossy();
                !ws_root_exclude.iter().any(|p| rel_str.starts_with(p.to_str().unwrap()))
            });
        for item in walker {
            let item = item?;
            let path = item.path();
            let rel_path = path.strip_prefix(ws_root)?;

            if rel_path.as_os_str().is_empty() {
                continue;
            }

            if path.is_file() {
                tar.append_path_with_name(path, rel_path)?;
            } else if path.is_dir() {
                tar.append_dir(rel_path, path)?;
            }
        }
        tar.finish()?;
        let buf: Vec<_> = tar.into_inner()?;
        let body = bollard::body_full(buf.into());
        let conf: bollard::query_parameters::BuildImageOptions = bollard::query_parameters::BuildImageOptionsBuilder::new()
            .dockerfile(ws_root_dockerfile_rel_path.to_str().unwrap())
            .t(image_tag)
            .rm(true)
            .pull("true")
            .build();
        let mut stream = self.build_image(conf, None, Some(body));
        while let Some(msg) = stream.try_next().await? {
            if let Some(s) = msg.stream {
                print!("{}", s);
            }
            if let Some(error) = msg.error_detail {
                return Err(format!("internal docker error: {:?}", error).into());
            }
        }
        let mut export_stream = self.export_image(image_tag);
        let mut file = std::fs::File::create(image_out_dir.join(image_name))?;
        while let Some(chunk) = export_stream.try_next().await? {
            file.write_all(&chunk)?;
        }
        file.flush()?;
        Ok(())
    }
}



async fn build_docker_image(
    docker: &bollard::Docker, 
    dockerfile_path: &std::path::Path, 
    path: &std::path::Path,
    image_file_path: &std::path::Path,
    image_name: String,
    image_tag: Option<String>
) -> Result<()> {
    let image_file_path_extension: &std::ffi::OsStr = image_file_path.extension().ok_or("image file path must have a `.tar` extension")?;
    if image_file_path_extension != "tar" {
        let error: Box<dyn std::error::Error> = "image file path must have a `.tar` extension".into();
        return Err(error)
    }

    let mut dockerfile = std::fs::File::open(dockerfile_path)?;
    let mut dockerfile_content = vec![];
    dockerfile.read_to_end(&mut dockerfile_content)?;
    let dockerfile_content_size = dockerfile_content.len() as u64;
    let mut dockerfile_header: tar::Header = tar::Header::new_gnu();
    dockerfile_header.set_path("Dockerfile")?;
    dockerfile_header.set_size(dockerfile_content_size);
    dockerfile_header.set_mode(0o644);
    dockerfile_header.set_cksum();

    let mut bin_file: std::fs::File = std::fs::File::open(path)?;
    let mut bin_content = vec![];
    bin_file.read_to_end(&mut bin_content)?;
    let bin_content_size = bin_content.len() as u64;
    let mut bin_header = tar::Header::new_gnu();
    bin_header.set_path("node")?;
    bin_header.set_size(bin_content_size);
    bin_header.set_mode(0o755);
    bin_header.set_cksum();
    
    let mut archive = tar::Builder::new(vec![]);
    archive.append(&dockerfile_header, &dockerfile_content[..])?;
    archive.append(&bin_header, &bin_content[..])?;

    let context: Vec<u8> = archive.into_inner()?;
    let context = bollard::body_full(context.into());
    let image_name: String = if let Some(image_tag) = image_tag {
        format!("{}:{}", image_name, image_tag)
    } else {
        image_name
    };
    let conf: bollard::query_parameters::BuildImageOptions = bollard::query_parameters::BuildImageOptionsBuilder::new()
        .dockerfile("Dockerfile")
        .t(&image_name)
        .rm(true)
        .build();
    let mut stream = docker.build_image(conf, None, Some(context));
    while let Some(msg) = stream.try_next().await? {
        if let Some(stream) = msg.stream {
            print!("{}", stream);
        }
        if let Some(error) = msg.error_detail {
            let error: Box<dyn std::error::Error> = format!("docker build failed: {:?}", error).into();
            return Err(error)
        }
    }
    println!("built image {}", image_name);
    let mut stream = docker.export_image(&image_name);
    let mut file: std::fs::File = std::fs::File::create(image_file_path)?;
    while let Some(bytes) = stream.try_next().await? {
        file.write_all(&bytes)?;
    }    
    Ok(())
}

fn workspace_dir() -> Result<std::path::PathBuf> {
    let output: std::process::Output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .output()?;
    if !output.status.success() {
        let error: Box<dyn std::error::Error> = "".into();
        return Err(error)
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let dir: std::path::PathBuf = metadata["workspace_root"]
        .as_str()
        .ok_or("missing `workspace_root`")?
        .into();
    Ok(dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    let main: Main = Main::parse();
    match &main.command {
        Command::BuildImage => {
            let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;
            let ws_root: std::path::PathBuf = workspace_dir()?;
            let ws_root_exclude: Vec<&str> = vec![
                ".github",
                ".obsidian",
                ".gitignore",
                "doc",
                "target",
                "task",
            ];
            let ws_root_exclude: Vec<std::path::PathBuf> = ws_root_exclude
                .iter()
                .map(|s| s.into())
                .collect();
            let roles: [_; _] = [
                "bootstrap",
                "client",
                "server",
                "relay"
            ];
            for role in roles {
                let ws_root: &std::path::Path = &ws_root;
                let ws_root_dockerfile_rel_path: std::path::PathBuf = format!("Dockerfile.{}", role).into();
                let ws_root_exclude: &[_] = &ws_root_exclude;
                let image_name: &str = role;
                let image_tag: &str = "latest";
                let image_out_dir: std::path::PathBuf = ws_root
                    .join("target")
                    .join("image");
                docker.snapshot(ws_root, &ws_root_dockerfile_rel_path, ws_root_exclude, image_name, image_tag, &image_out_dir).await?;
            }
        },
        Command::BuildNode => {
            let roles: [_; _] = [
                "bootstrap",
                "client",
                "server",
                "relay"
            ];
            for role in roles {
                std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--package")
                    .arg("node")
                    .arg("--bin")
                    .arg(role)
                    .arg(format!("--features={}", role))
                    .arg("--no-default-features")
                    .spawn()?
                    .wait()?;
            }
        },
        Command::BuildNodeRelease => {
            let roles: [_; _] = [
                "bootstrap",
                "client",
                "server",
                "relay"
            ];
            for role in roles {
                std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--release")
                    .arg("--package")
                    .arg("node")
                    .arg("--bin")
                    .arg(role)
                    .arg(format!("--features={}", role))
                    .arg("--no-default-features")
                    .spawn()?
                    .wait()?;
            }
        }
    }
    Ok(())
}