use crate::downloader_trait::Downloader;
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::stream_extractor::HARDCODED_CLIENT_VERSION;
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct YTTrendingExtractor {
    initial_data: Value,
}

impl YTTrendingExtractor {
    async fn get_initial_data<D: Downloader>(downloader: &D) -> Result<Value, ParsingError> {
        let url = format!("https://www.youtube.com/feed/trending?pbj=1");
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let url = format!("{}&pbj=1", url);
        let data = downloader.download_with_header(&url, headers).await?;
        let mut json =
            serde_json::from_str::<Value>(&data).map_err(|e| ParsingError::from(e.to_string()))?;
        Ok(json
            .get_mut(1)
            .ok_or("No index 1")?
            .get_mut("response")
            .ok_or("No response")?
            .take())
    }

    pub async fn new<D: Downloader>(downloader: D) -> Result<Self, ParsingError> {
        let initial_data = YTTrendingExtractor::get_initial_data(&downloader).await?;
        Ok(Self { initial_data })
    }
}

impl YTTrendingExtractor {
    pub fn get_videos(&self) -> Result<Vec<YTStreamInfoItemExtractor>, ParsingError> {
        let item_section_renderers = (|| {
            self.initial_data
                .get("contents")?
                .get("twoColumnBrowseResultsRenderer")?
                .get("tabs")?
                .get(0)?
                .get("tabRenderer")?
                .get("content")?
                .get("sectionListRenderer")?
                .get("contents")?
                .as_array()
        })()
        .ok_or("No item sections")?;
        let mut videos = vec![];
        for item_section in item_section_renderers {
            let shelf_content = (|| {
                item_section
                    .get("itemSectionRenderer")?
                    .get("contents")?
                    .get(0)?
                    .get("shelfRenderer")?
                    .get("content")?
                    .get("expandedShelfContentsRenderer")?
                    .get("items")?
                    .as_array()
            })();
            if let Some(shelf_content) = shelf_content {
                for ul in shelf_content {
                    if let Some(videoRenderer) =
                        ul.get("videoRenderer").unwrap_or(&Value::Null).as_object()
                    {
                        videos.push(YTStreamInfoItemExtractor {
                            video_info: videoRenderer.clone(),
                        })
                    }
                }
            }
        }
        Ok(videos)
    }
}
