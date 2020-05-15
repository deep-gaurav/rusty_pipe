use super::super::downloader_trait::Downloader;
use crate::youtube_extractor::channel_info_item_extractor::YTChannelInfoItemExtractor;
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::playlist_info_item_extractor::YTPlaylistInfoItemExtractor;
use crate::youtube_extractor::stream_extractor::HARDCODED_CLIENT_VERSION;
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use failure::Error;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use scraper::{ElementRef, Html, Selector};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Clone)]
pub enum YTSearchItem {
    StreamInfoItem(YTStreamInfoItemExtractor),
    ChannelInfoItem(YTChannelInfoItemExtractor),
    PlaylistInfoItem(YTPlaylistInfoItemExtractor),
}

pub struct YTSearchExtractor {
    initial_data: Map<String, Value>,
    query: String,
    page: Option<(Vec<YTSearchItem>, Option<String>)>,
}

impl YTSearchExtractor {
    async fn get_initial_data<D: Downloader>(
        downloader: &D,
        url: &str,
    ) -> Result<Map<String, Value>, ParsingError> {
        let url = format!("{}&gl=IN&pbj=1", url);
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let resp = downloader.download_with_header(&url, headers).await?;
        let resp_json = serde_json::from_str::<Value>(&resp)
            .map_err(|er| ParsingError::parsing_error_from_str(&er.to_string()))?;
        let resp_json = resp_json
            .get(1)
            .ok_or("index 1 not in pbj")?
            .get("response")
            .ok_or("response not in pbj")?
            .as_object()
            .ok_or(format!("initial data not json object "))?
            .to_owned();
        Ok(resp_json)
    }

    fn collect_streams_from(videos: &Vec<Value>) -> Result<Vec<YTSearchItem>, ParsingError> {
        let mut search_items = vec![];
        for item in videos {
            if item.get("backgroundPromoRenderer").is_some() {
                return Err(ParsingError::from("Nothing found"));
            }
            if let Some(el) = item.get("videoRenderer").map(|f| f.as_object()) {
                if let Some(vid_info) = el {
                    search_items.push(YTSearchItem::StreamInfoItem(YTStreamInfoItemExtractor {
                        video_info: vid_info.to_owned(),
                    }));
                }
            } else if let Some(el) = item.get("channelRenderer").map(|f| f.as_object()) {
                if let Some(vid_info) = el {
                    search_items.push(YTSearchItem::ChannelInfoItem(YTChannelInfoItemExtractor {
                        channel_info: vid_info.to_owned(),
                    }));
                }
            } else if let Some(el) = item.get("playlistRenderer").map(|f| f.as_object()) {
                if let Some(vid_info) = el {
                    search_items.push(YTSearchItem::PlaylistInfoItem(
                        YTPlaylistInfoItemExtractor {
                            playlist_info: vid_info.to_owned(),
                        },
                    ));
                }
            }
        }

        Ok(search_items)
    }

    fn get_next_page_url_from(continuation: &Value, query: &str) -> Option<String> {
        let next_continuation_data = continuation.get(0)?.get("nextContinuationData")?;
        let continuation = next_continuation_data.get("continuation")?.as_str()?;
        let click_tracking_params = next_continuation_data
            .get("clickTrackingParams")?
            .as_str()?;
        Some(format!(
            "https://www.youtube.com/results?pbj=1&search_query={}&ctoken={}&continuation={}&itct={}",
            query,continuation, continuation, click_tracking_params
        ))
    }

    async fn get_page<D: Downloader>(
        page_url: &str,
        downloader: &D,
        query: &str,
    ) -> Result<(Vec<YTSearchItem>, Option<String>), ParsingError> {
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let response = downloader.download_with_header(&page_url, headers).await?;
        let json_response = serde_json::from_str::<Value>(&response)
            .map_err(|e| ParsingError::from(format!("json eror : {:#?}", e)))?;

        let section_list_continuation = (|| {
            json_response
                .get(1)?
                .get("response")?
                .get("continuationContents")?
                .get("itemSectionContinuation")
        })()
        .ok_or("Cant get continuation")?;

        let items = YTSearchExtractor::collect_streams_from(
            section_list_continuation
                .get("contents")
                .ok_or("Not contents")?
                .as_array()
                .ok_or("items not in continuation")?,
        )?;
        let next_url = YTSearchExtractor::get_next_page_url_from(
            section_list_continuation
                .get("continuations")
                .unwrap_or(&Value::Null),
            query,
        );

        Ok((items, next_url))
    }
}

impl YTSearchExtractor {
    pub async fn new<D: Downloader>(
        downloader: D,
        query: &str,
        page_url: Option<String>,
    ) -> Result<YTSearchExtractor, ParsingError> {
        let url = format!(
            "https://www.youtube.com/results?disable_polymer=1&search_query={}",
            query
        );
        let query = utf8_percent_encode(query, FRAGMENT).to_string();
        if let Some(page_url) = page_url {
            let initial_data = YTSearchExtractor::get_initial_data(&downloader, &url);
            let page = YTSearchExtractor::get_page(&page_url, &downloader, &query);
            use futures::try_join;
            let (initial_data, page) = try_join!(initial_data, page)?;

            Ok(YTSearchExtractor {
                initial_data,
                query,
                page: Some(page),
            })
        } else {
            let initial_data = YTSearchExtractor::get_initial_data(&downloader, &url).await?;
            Ok(YTSearchExtractor {
                initial_data,
                query,
                page: None,
            })
        }
    }

    pub fn get_search_suggestion(&self) -> Result<String, ParsingError> {
        let showing_results: Value = (|initdata: &Map<String, Value>| {
            let data = initdata
                .get("contents")?
                .get("twoColumnSearchResultsRenderer")?
                .get("primaryContents")?
                .get("sectionListRenderer")?
                .get("contents")?
                .get(0)?
                .get("itemSectionRenderer")?
                .get("contents")?
                .get(0)?
                .get("showingResultsForRenderer")?;
            Some(data.to_owned())
        })(&self.initial_data)
        .unwrap_or_default();
        if let Some(correction) = showing_results.get("correctedQuery") {
            if let Some(correction) = correction.as_str() {
                return Ok(correction.to_string());
            }
        }
        Ok("".to_string())
    }

    pub fn search_results(&self) -> Result<Vec<YTSearchItem>, ParsingError> {
        if let Some((items, _)) = &self.page {
            return Ok(items.clone());
        }
        // println!("{:#?}",self.initial_data);
        let sections = (|| {
            let data = self
                .initial_data
                .get("contents")?
                .get("twoColumnSearchResultsRenderer")?
                .get("primaryContents")?
                .get("sectionListRenderer")?
                .get("contents")?
                .as_array()?;
            Some(data)
        })()
        .ok_or("cant get sections ")?;

        let mut search_items: Vec<YTSearchItem> = vec![];

        for sect in sections {
            let item_section = (|| {
                let c = sect
                    .get("itemSectionRenderer")?
                    .get("contents")?
                    .as_array()?;
                Some(c)
            })()
            .ok_or("cant get section")?;

            search_items.append(&mut YTSearchExtractor::collect_streams_from(&item_section)?)
        }
        return Ok(search_items);
    }

    pub fn get_next_page_url(&self) -> Result<Option<String>, ParsingError> {
        if let Some((_, url)) = &self.page {
            return Ok(url.clone());
        } else {
            return Ok(YTSearchExtractor::get_next_page_url_from(
                (|| {
                    self.initial_data
                        .get("contents")?
                        .get("twoColumnSearchResultsRenderer")?
                        .get("primaryContents")?
                        .get("sectionListRenderer")?
                        .get("contents")?
                        .get(0)?
                        .get("itemSectionRenderer")?
                        .get("continuations")
                })()
                .ok_or("no continuation")?,
                &self.query,
            ));
        }
    }
}
