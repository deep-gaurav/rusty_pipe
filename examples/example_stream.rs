extern crate rusty_pipe;

use rusty_pipe::youtube_extractor::search_extractor::{*};
use rusty_pipe::youtube_extractor::stream_extractor::{*};
use rusty_pipe::downloader_trait::Downloader;
use std::io;
use scraper::{Html};

use urlencoding::encode;
use async_trait::async_trait;


#[tokio::main]
async fn main() -> Result<(), String> {

    static APP_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:68.0) Gecko/20100101 Firefox/43.0";



    // let url = "https://www.youtube.com/watch?v=09R8_2nJtjg&disable_polymer=1";
    // let resp = reqwest::get(url).await.map_err(|er|er.to_string())?;

    let downloader = DownloaderExample{};
    // let body = downloader.download(url).await?;

    let mut stream_extractor = YTStreamExtractor::new("09R8_2nJtjg", downloader).await?;
    let video_streams = stream_extractor.get_video_stream().await?;
    println!("AUDIO/VIDEO STREAMS \n");
    println!("{:#?}",video_streams);

    let audio_streams = stream_extractor.get_audio_streams().await?;
    println!("AUDIO ONLY STREAMS \n");
    println!("{:#?}",audio_streams);

    let video_only_streams = stream_extractor.get_video_only_stream().await?;
    println!("VIDEO ONLY STREAMS \n");
    println!("{:#?}",video_only_streams);

    Ok(())
}

struct DownloaderExample{

}

#[async_trait]
impl Downloader for DownloaderExample{
    async fn download(&self, url: &str) -> Result<String, String> {
        println!("query url : {}",url);
        let resp = reqwest::get(url).await.map_err(|er|er.to_string())?;
        println!("got response ");
        let body = resp.text().await.map_err(|er|er.to_string())?;
        println!("suceess query");
        Ok(String::from(body))
    }
}