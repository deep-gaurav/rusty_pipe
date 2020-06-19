use crate::youtube_extractor::error::ParsingError;
use async_trait::async_trait;
use failure::Error;
use std::collections::HashMap;

#[async_trait]
pub trait Downloader {
    async fn download(url: &str) -> Result<String, ParsingError>;
    async fn download_with_header(
        url: &str,
        header: HashMap<String, String>,
    ) -> Result<String, ParsingError>;
    fn eval_js(script: &str) -> Result<String, String>;
}
