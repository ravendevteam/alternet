use super::*;

pub struct Alpine;

#[async_trait::async_trait]
impl Image for Alpine {
    async fn render() -> Result<GenericImage> {
        GenericBuildableImage::new("alpine", "tag")
            .with_dockerfile_string(
                r#"
                    FROM alpine:latest
                "#
            )
            .build_image()
            .await
            .map_err(testcontainers::TestcontainersError::into)
    }
}