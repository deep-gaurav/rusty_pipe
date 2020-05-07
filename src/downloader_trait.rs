use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait Downloader {
    async fn download(&self, url: &str) -> Result<String, String>;
    async fn download_with_header(&self, url:&str, header:HashMap<String,String>) -> Result<String,String>;
}
