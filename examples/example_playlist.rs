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
use rusty_pipe::youtube_extractor::playlist_extractor::YTPlaylistExtractor;
use rusty_pipe::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use serde_json::Value;

struct DownloaderExample;

#[async_trait]
impl Downloader for DownloaderExample {
    async fn download( url: &str) -> Result<String, ParsingError> {
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

    async fn download_with_header(
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

    fn eval_js(script: &str) -> Result<String, String> {
        use quick_js::{Context, JsValue};
        let context = Context::new().expect("Cant create js context");
        // println!("decryption code \n{}",decryption_code);
        // println!("signature : {}",encrypted_sig);
        println!("jscode \n{}",script);
        let res = context.eval(script).unwrap_or(quick_js::JsValue::Null);
        // println!("js result : {:?}", result);
        let result = res.into_string().unwrap_or("".to_string());
        print!("JS result: {}",result);
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
    println!("Enter playlist id: ");
    let mut playlist_id = String::new();
    std::io::stdin()
        .read_line(&mut playlist_id)
        .expect("Input failed");
    playlist_id = playlist_id.trim().to_string();
    let playlist_extractor =
        YTPlaylistExtractor::new(&playlist_id, DownloaderExample, None).await?;
    println!("Playlist name {:#?}", playlist_extractor.get_name());
    println!(
        "Playlist Thumbnails \n{:#?}",
        playlist_extractor.get_thumbnails()
    );
    println!(
        "Uploader name: {:#?}",
        playlist_extractor.get_uploader_name()
    );
    println!("Uploader url: {:#?}", playlist_extractor.get_uploader_url());
    println!(
        "Uploaders thumbnails \n{:#?}",
        playlist_extractor.get_uploader_avatars()
    );

    println!(
        "Videos count : {:#?}",
        playlist_extractor.get_stream_count()
    );

    println!("Videos :\n");
    // print_videos(channel_extractor.get_videos()?);
    let mut videos = vec![];
    videos.append(&mut playlist_extractor.get_videos()?);
    println!(
        "Next Page url: {:#?}",
        playlist_extractor.get_next_page_url()
    );

    let mut next_page_url = playlist_extractor.get_next_page_url()?;

    while let Some(next_page) = next_page_url.clone() {
        let extractor =
            YTPlaylistExtractor::new(&playlist_id, DownloaderExample, Some(next_page)).await?;
        // print_videos(extractor.get_videos()?);
        next_page_url = extractor.get_next_page_url()?;
        videos.append(&mut playlist_extractor.get_videos()?);
        println!("Next page url {:#?}", next_page_url);
    }
    print_videos(videos);

    Ok(())
}
