use crate::downloader_trait::Downloader;
use crate::utils::utils::{fix_thumbnail_url, get_text_from_object};
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::stream_extractor::{Thumbnail, HARDCODED_CLIENT_VERSION};
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use futures::try_join;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub static CHANNEL_URL_BASE: &str = "https://www.youtube.com/channel/";

#[derive(Clone, PartialEq)]
pub struct YTChannelExtractor {
    initial_data: Value,
    video_tab: Value,
    page: Option<(Vec<YTStreamInfoItemExtractor>, Option<String>)>,
}

impl YTChannelExtractor {
    async fn get_initial_data<D: Downloader>(id: &str) -> Result<Value, ParsingError> {
        let mut url = format!("{}{}/videos?pbj=1&view=0&flow=grid", CHANNEL_URL_BASE, id);

        let mut level = 0;
        let mut ajax_json = Value::Null;
        while level < 3 {
            let mut headers = HashMap::new();
            headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
            headers.insert(
                "X-YouTube-Client-Version".to_string(),
                HARDCODED_CLIENT_VERSION.to_string(),
            );
            let response = D::download_with_header(&url, headers).await?;
            let json_response = serde_json::from_str::<Value>(&response)
                .map_err(|e| ParsingError::from(e.to_string()))?;
            let endpoint = (|| {
                json_response
                    .get(1)?
                    .get("response")?
                    .get("onResponseReceivedActions")?
                    .get(0)?
                    .get("navigateAction")?
                    .get("endpoint")
            })()
            .unwrap_or(&Value::Null);
            let webpage_type = (|| {
                endpoint
                    .get("commandMetadata")?
                    .get("webCommandMetadata")?
                    .get("webPageType")?
                    .as_str()
            })()
            .unwrap_or_default();
            let browse_id = (|| endpoint.get("browseEndpoint")?.get("browseId")?.as_str())()
                .unwrap_or_default();

            if webpage_type.eq_ignore_ascii_case("WEB_PAGE_TYPE_BROWSE") && !browse_id.is_empty() {
                if !browse_id.starts_with("UC") {
                    return Err(ParsingError::from(
                        "Redirected id is not pointing to a channel",
                    ));
                }
                url = format!(
                    "https://www.youtube.com/channel/{}/videos?pbj=1&view=0&flow=grid",
                    browse_id
                );
                level += 1;
            } else {
                ajax_json = json_response;
                break;
            }
        }

        if ajax_json == Value::Null {
            Err(ParsingError::from("Could not fetch initial JSON data"))
        } else {
            let init_data =
                (|| ajax_json.get(1)?.get("response"))().ok_or("reponse null in ajax json")?;
            Ok(init_data.clone())
        }
    }

    fn get_video_tab(initial_data: &Value) -> Result<Value, ParsingError> {
        let tabs = (|| {
            initial_data
                .get("contents")?
                .get("twoColumnBrowseResultsRenderer")?
                .get("tabs")?
                .as_array()
        })()
        .ok_or("Tabs not found")?;
        let mut video_tab = &Value::Null;

        for tab in tabs {
            if let Some(renderer) = tab.get("tabRenderer") {
                if renderer
                    .get("title")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or_default()
                    == "Videos"
                {
                    video_tab = renderer;
                    break;
                }
            }
        }

        if video_tab == &Value::Null {
            return Err(ParsingError::from("This channel has no Videos tab"));
        }
        let message_renderer_text = (|| {
            Some(get_text_from_object(
                video_tab
                    .get("content")?
                    .get("sectionListRenderer")?
                    .get("contents")?
                    .get(0)?
                    .get("itemSectionRenderer")?
                    .get("contents")?
                    .get("0")?
                    .get("messageRenderer")?
                    .get("text")?,
                false,
            ))
        })();

        if let Some(message) = message_renderer_text {
            if let Some(message) = message? {
                if message == "This channel has no videos." {
                    return Ok(Value::Null);
                }
            }
        }
        Ok(video_tab.clone())
    }

    pub async fn new<D: Downloader>(
        channel_id: &str,
        page_url: Option<String>,
    ) -> Result<Self, ParsingError> {
        if let Some(page_url) = page_url {
            let initial_data = YTChannelExtractor::get_initial_data::<D>(channel_id);
            let page = YTChannelExtractor::get_page::<D>(&page_url);
            use futures::try_join;
            let (initial_data, page) = try_join!(initial_data, page)?;
            let video_tab = YTChannelExtractor::get_video_tab(&initial_data)?;

            Ok(YTChannelExtractor {
                initial_data,
                video_tab,
                page: Some(page),
            })
        } else {
            let initial_data = YTChannelExtractor::get_initial_data::<D>(channel_id).await?;
            let video_tab = YTChannelExtractor::get_video_tab(&initial_data)?;
            Ok(YTChannelExtractor {
                initial_data,
                video_tab,
                page: None,
            })
        }
    }

    fn collect_streams_from(
        videos: &Value,
    ) -> Result<Vec<YTStreamInfoItemExtractor>, ParsingError> {
        let mut streams = vec![];
        for video in videos.as_array().ok_or("videos not array")? {
            if let Some(vid_renderer) = video.get("gridVideoRenderer") {
                if let Value::Object(video_info) = vid_renderer {
                    streams.push(YTStreamInfoItemExtractor {
                        video_info: video_info.clone(),
                    });
                }
            }
        }

        Ok(streams)
    }

    fn get_next_page_url_from(continuation: &Value) -> Option<String> {
        let next_continuation_data = continuation.get(0)?.get("nextContinuationData")?;
        let continuation = next_continuation_data.get("continuation")?.as_str()?;
        let click_tracking_params = next_continuation_data
            .get("clickTrackingParams")?
            .as_str()?;
        Some(format!(
            "https://www.youtube.com/browse_ajax?ctoken={}&continuation={}&itct={}",
            continuation, continuation, click_tracking_params
        ))
    }

    async fn get_page<D: Downloader>(
        page_url: &str,
    ) -> Result<(Vec<YTStreamInfoItemExtractor>, Option<String>), ParsingError> {
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let response = D::download_with_header(&page_url, headers).await?;
        let json_response = serde_json::from_str::<Value>(&response)
            .map_err(|e| ParsingError::from(e.to_string()))?;

        let section_list_continuation = (|| {
            json_response
                .get(1)?
                .get("response")?
                .get("continuationContents")?
                .get("gridContinuation")
        })()
        .ok_or("Cant get continuation")?;

        let items = YTChannelExtractor::collect_streams_from(
            section_list_continuation
                .get("items")
                .ok_or("items not in continuation")?,
        )?;
        let next_url = YTChannelExtractor::get_next_page_url_from(
            section_list_continuation
                .get("continuations")
                .unwrap_or(&Value::Null),
        );

        Ok((items, next_url))
    }
}

impl YTChannelExtractor {
    pub fn get_name(&self) -> Result<String, ParsingError> {
        Ok((|| {
            self.initial_data
                .get("header")?
                .get("c4TabbedHeaderRenderer")?
                .get("title")?
                .as_str()
        })()
        .ok_or("Cant get title")?
        .to_string())
    }

    pub fn get_avatars(&self) -> Result<Vec<Thumbnail>, ParsingError> {
        let mut thumbnails = vec![];
        for thumb in self
            .initial_data
            .get("header")
            .ok_or("No header")?
            .get("c4TabbedHeaderRenderer")
            .ok_or("No c4tabbed header")?
            .get("avatar")
            .ok_or("no avatar")?
            .get("thumbnails")
            .ok_or("no thumbnails")?
            .as_array()
            .ok_or("thumbnails array")?
        {
            // println!("{:#?}",thumb);
            if let Ok(thumb) = serde_json::from_value(thumb.to_owned()) {
                // thumb.url = fix_thumbnail_url(&thumb.url);
                thumbnails.push(thumb)
            }
        }
        Ok(thumbnails)
    }
    pub fn get_banners(&self) -> Result<Vec<Thumbnail>, ParsingError> {
        let mut thumbnails = vec![];
        for thumb in self
            .initial_data
            .get("header")
            .ok_or("No header")?
            .get("c4TabbedHeaderRenderer")
            .ok_or("No c4tabbed header")?
            .get("banner")
            .ok_or("no banner")?
            .get("thumbnails")
            .ok_or("no thumbnails")?
            .as_array()
            .ok_or("thumbnails array")?
        {
            // println!("{:#?}",thumb);
            if let Ok(thumb) = serde_json::from_value(thumb.to_owned()) {
                // thumb.url = fix_thumbnail_url(&thumb.url);
                thumbnails.push(thumb)
            }
        }
        Ok(thumbnails)
    }

    pub fn get_videos(&self) -> Result<Vec<YTStreamInfoItemExtractor>, ParsingError> {
        if let Some((videos, _)) = &self.page {
            return Ok(videos.clone());
        }
        let videos = (|| {
            self.video_tab
                .get("content")?
                .get("sectionListRenderer")?
                .get("contents")?
                .get(0)?
                .get("itemSectionRenderer")?
                .get("contents")?
                .get(0)?
                .get("gridRenderer")?
                .get("items")
        })()
        .ok_or("Cant get videos")?;
        YTChannelExtractor::collect_streams_from(videos)
    }

    pub fn get_next_page_url(&self) -> Result<Option<String>, ParsingError> {
        if let Some((_, page_url)) = &self.page {
            Ok(page_url.clone())
        } else {
            let conti = (|| {
                self.video_tab
                    .get("content")?
                    .get("sectionListRenderer")?
                    .get("contents")?
                    .get(0)?
                    .get("itemSectionRenderer")?
                    .get("contents")?
                    .get(0)?
                    .get("gridRenderer")?
                    .get("continuations")
            })();
            if let Some(conti) = conti {
                Ok(YTChannelExtractor::get_next_page_url_from(conti))
            } else {
                println!("Continuation is None");
                Ok(None)
            }
        }
    }
}
