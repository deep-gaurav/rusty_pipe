use crate::youtube_extractor::channel_info_item_extractor::YTChannelInfoItemExtractor;
use crate::youtube_extractor::playlist_info_item_extractor::YTPlaylistInfoItemExtractor;
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use scraper::{ElementRef, Html, Selector};
use super::super::downloader_trait::Downloader;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use failure::Error;
use serde_json::{Value, Map};
use std::collections::HashMap;
use crate::youtube_extractor::stream_extractor::HARDCODED_CLIENT_VERSION;
use crate::youtube_extractor::error::ParsingError;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

pub enum YTSearchItem {
    StreamInfoItem(YTStreamInfoItemExtractor),
    ChannelInfoItem(YTChannelInfoItemExtractor),
    PlaylistInfoItem(YTPlaylistInfoItemExtractor),
}

pub struct YTSearchExtractor {
    initial_data:Map<String,Value>,
}


impl YTSearchExtractor{
    async fn get_initial_data<D:Downloader>(downloader:&D,url:&str)->Result<Map<String,Value>,ParsingError>{
        let url = format!("{}&gl=IN&pbj=1",url);
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let resp = downloader.download_with_header(&url,headers).await?;
        let resp_json = serde_json::from_str::<Value>(&resp)
            .map_err(|er|ParsingError::parsing_error_from_str(&er.to_string()))?;
        let resp_json = resp_json
            .get(1)
            .ok_or("index 1 not in pbj")?
            .get("response")
            .ok_or("response not in pbj")?
            .as_object()
            .ok_or(format!("initial data not json object " ))
            ?.to_owned();
        Ok(resp_json)
    }
}

impl YTSearchExtractor {

    pub async fn new<D:Downloader>(downloader:D,query:&str)->Result<YTSearchExtractor,ParsingError>{
        let query = utf8_percent_encode(query,FRAGMENT).to_string();
        let url = format!(
            "https://www.youtube.com/results?disable_polymer=1&search_query={}",
            query
        );
        let initial_data = YTSearchExtractor::get_initial_data(&downloader,&url).await?;



        Ok(
            YTSearchExtractor{
                initial_data
            }
        )

    }

    pub fn get_search_suggestion(&self) -> Result<String,ParsingError> {
        let showing_results:Option<Value>= (|initdata:&Map<String,Value>| {

                let data = initdata.get("contents")?
                    .get("twoColumnSearchResultsRenderer")?.get("primaryContents")?
                    .get("sectionListRenderer")?.get("contents")?.get(0)?
                    .get("itemSectionRenderer")?.get("contents")?.get(0)?
                    .get("showingResultsForRenderer")?;
                Some(data.to_owned())
            })(&self.initial_data);
        Ok("".to_string())
    }

    pub fn collect_items(&self) -> Result<Vec<YTSearchItem>,ParsingError> {
        // println!("{:#?}",self.initial_data);
        let sections = (||{
            let data = self.initial_data.get("contents")?
                .get("twoColumnSearchResultsRenderer")?.get("primaryContents")?
                .get("sectionListRenderer")?.get("contents")?.as_array()?;
            Some(data)
        })().ok_or("cant get sections ")?;

        let mut search_items: Vec<YTSearchItem> = vec![];

        for sect in sections{
            let item_section = (||{
                let c = sect.get("itemSectionRenderer")?.get("contents")?.as_array()?;
                Some(c)
            })().ok_or("cant get section")?;

            for item in item_section {
                if item.get("backgroundPromoRenderer").is_some()
                {
                    return Err(ParsingError::from("Nothing found"));
                }
                if let Some(el) = item.get("videoRenderer").map(|f|f.as_object())
                {
                    if let Some(vid_info) = el{

                        search_items.push(YTSearchItem::StreamInfoItem(YTStreamInfoItemExtractor {
                            video_info:vid_info.to_owned(),
                        }));
                    }
                }
                else if let Some(el) = item.get("channelRenderer").map(|f|f.as_object())
                {
                    if let Some(vid_info) = el{

                        search_items.push(YTSearchItem::ChannelInfoItem(YTChannelInfoItemExtractor {
                            channel_info:vid_info.to_owned(),
                        }));
                    }
                }

                else if let Some(el) = item.get("playlistRenderer").map(|f|f.as_object())
                {
                    if let Some(vid_info) = el{

                        search_items.push(YTSearchItem::PlaylistInfoItem(YTPlaylistInfoItemExtractor {
                            playlist_info:vid_info.to_owned(),
                        }));
                    }
                }

            }
        }
        return Ok(search_items);
    }

}
