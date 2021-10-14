extern crate rusty_pipe;

use rusty_pipe::youtube_extractor::search_extractor::*;
use std::io;

use rusty_pipe::downloader_trait::Downloader;
use std::collections::HashMap;
use std::str::FromStr;
use urlencoding::encode;

use async_trait::async_trait;
use failure::Error;
use rusty_pipe::youtube_extractor::channel_extractor::YTChannelExtractor;
use rusty_pipe::youtube_extractor::error::ParsingError;
use rusty_pipe::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use serde_json::Value;

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
    println!("Enter channel id: ");
    let mut channel_id = String::new();
    std::io::stdin()
        .read_line(&mut channel_id)
        .expect("Input failed");
    channel_id = channel_id.trim().to_string();
    let channel_extractor = YTChannelExtractor::new(DownloaderExample,&channel_id, None).await?;
    println!("Channel name {:#?}", channel_extractor.get_name());
    println!(
        "Channel Thumbnails \n{:#?}",
        channel_extractor.get_avatars()
    );
    println!("Channel Banners \n{:#?}", channel_extractor.get_banners());
    println!("Videos :\n");
    // print_videos(channel_extractor.get_videos()?);
    let mut videos = vec![];
    videos.append(&mut channel_extractor.get_videos()?);
    println!(
        "Next Page url: {:#?}",
        channel_extractor.get_next_page_url()
    );

    let mut next_page_url = channel_extractor.get_next_page_url()?;

    while let Some(next_page) = next_page_url.clone() {
        let extractor =
            YTChannelExtractor::new(DownloaderExample, &channel_id, Some(next_page)).await?;
        // print_videos(extractor.get_videos()?);
        next_page_url = extractor.get_next_page_url()?;
        videos.append(&mut channel_extractor.get_videos()?);
        println!("Next page url {:#?}", next_page_url);
    }
    print_videos(videos);

    Ok(())
}
