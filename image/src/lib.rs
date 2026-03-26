use testcontainers::runners::AsyncBuilder as _;
use testcontainers::runners::AsyncRunner as _;

pub mod alpine;
pub mod node;
pub mod router;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("{}", 0)]
    TestContainer(#[from] testcontainers::TestcontainersError),
    #[error("{}", 0)]
    Metadata(#[from] cargo_metadata::Error)
}

pub type GenericBuildableImage = testcontainers::GenericBuildableImage;
pub type GenericImage = testcontainers::GenericImage;

#[async_trait::async_trait]
pub trait Image {
    async fn render() -> Result<GenericImage>;
}

fn ws_root() -> Result<std::path::PathBuf> {
    let metadata: cargo_metadata::Metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    let ret: std::path::PathBuf = metadata.workspace_root.into_std_path_buf();
    Ok(ret)
}