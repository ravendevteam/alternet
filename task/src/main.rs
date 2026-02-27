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

    let mut bin_file = std::fs::File::open(path)?;
    let mut bin_content = vec![];
    bin_file.read_to_end(&mut bin_content)?;
    let bin_content_size = bin_content.len() as u64;
    let mut bin_header = tar::Header::new_gnu();
    bin_header.set_path("app")?;
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
            let bins: Vec<_> = bin_to_crate()?;
            let target_dir: std::path::PathBuf = workspace_dir()?.join("target");
            if !target_dir.exists() {
                std::fs::create_dir(&target_dir)?;
            }
            let image_dir: std::path::PathBuf = target_dir.join("image");
            if !image_dir.exists() {
                std::fs::create_dir(&image_dir)?;
            }
            let release_dir: std::path::PathBuf = target_dir.join("release");
            if !release_dir.exists() {
                return Ok(())
            }
            for (bin_name, crate_dir) in bins {
                let dockerfile_path: std::path::PathBuf = crate_dir.join("Dockerfile");
                if !dockerfile_path.exists() {
                    continue
                }
                let bin_path: std::path::PathBuf = release_dir.join(&bin_name);
                if !bin_path.exists() {
                    eprintln!("release binary not found: {:?}", bin_path);
                    continue
                }
                let image_tar_path: String = format!("{}.tar", bin_name);
                let image_tar_path: std::path::PathBuf = image_dir.join(image_tar_path);
                let image_name: String = bin_name;
                build_docker_image(&docker, &dockerfile_path, &bin_path, &image_tar_path, image_name, None).await?;
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