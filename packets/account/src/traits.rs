#[async_trait::async_trait]
pub trait Authanticator {
    async fn auth(username: &str, password: &str) -> Result<u32, crate::Error>;
}
