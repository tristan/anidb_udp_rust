use async_trait::async_trait;

#[async_trait]
pub trait AniDbCache {
    type Error: std::error::Error + std::fmt::Debug + std::fmt::Display;
    async fn get(
        &self,
        command: &str,
        args: &str
    ) -> Result<Option<(String, String, String)>, Self::Error>;
    async fn store(
        &self,
        command: &str,
        args: &str,
        code: &str,
        reply: &str,
        data: &str
    ) -> Result<(), Self::Error>;
}
