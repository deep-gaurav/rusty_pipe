extern crate rusty_pipe;

use async_trait::async_trait;
use failure::Error;
use rusty_pipe::downloader_trait::Downloader;
use rusty_pipe::youtube_extractor::error::ParsingError;
use rusty_pipe::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use rusty_pipe::youtube_extractor::trending_extractor::YTTrendingExtractor;
use std::collections::HashMap;
use std::str::FromStr;

struct DownloaderExample;

#[async_trait]
impl Downloader for DownloaderExample {
    async fn download(&self,url: &str) -> Result<String, ParsingError> {
        println!("query url : {}", url);
        let resp = reqwest::get(url)
            .await
            .map_err(|er| ParsingError::DownloadError {
                cause: er.to_string(),
            })?;
        println!("got response ");
        let body = resp
            .text()
            .await
            .map_err(|er| ParsingError::DownloadError {
                cause: er.to_string(),
            })?;
        println!("suceess query");
        Ok(String::from(body))
    }

    async fn download_with_header(&self,
        url: &str,
        header: HashMap<String, String>,
    ) -> Result<String, ParsingError> {
        let client = reqwest::Client::new();
        let res = client.get(url);
        let mut headers = reqwest::header::HeaderMap::new();
        for header in header {
            headers.insert(
                reqwest::header::HeaderName::from_str(&header.0).map_err(|e| e.to_string())?,
                header.1.parse().unwrap(),
            );
        }
        let res = res.headers(headers);
        let res = res.send().await.map_err(|er| er.to_string())?;
        let body = res.text().await.map_err(|er| er.to_string())?;
        Ok(String::from(body))
    }

    async fn eval_js(&self,script: &str) -> Result<String, String> {
        use quick_js::{Context, JsValue};
        let context = Context::new().expect("Cant create js context");
        // println!("decryption code \n{}",decryption_code);
        // println!("signature : {}",encrypted_sig);
        println!("jscode \n{}", script);
        let res = context.eval(script).unwrap_or(quick_js::JsValue::Null);
        // println!("js result : {:?}", result);
        let result = res.into_string().unwrap_or("".to_string());
        print!("JS result: {}", result);
        Ok(result)
    }
}

fn print_videos(videos: Vec<YTStreamInfoItemExtractor>) {
    let mut count = 0;
    for vid in videos {
        count += 1;
        println!("STREAM {}", count);
        println!("title: {:#?}", vid.get_name());
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let extractor = YTTrendingExtractor::new(DownloaderExample).await?;

    let videos = extractor.get_videos()?;

    print_videos(videos);
    Ok(())
}
