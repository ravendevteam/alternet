type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let out_dir: std::path::PathBuf = "proto_target".into();
    if !out_dir.exists() {
        std::fs::create_dir(&out_dir)?;
    }
    println!("cargo:rerun-if-changed=proto/an.proto");
    let proto_paths: Vec<_> = vec!["proto/an.proto"];
    let proto_inclusions: Vec<_> = vec!["proto"];
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(out_dir)
        .compile_protos(&proto_paths, &proto_inclusions)?;
    Ok(())
}