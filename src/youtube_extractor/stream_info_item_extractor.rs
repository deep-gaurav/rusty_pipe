use crate::utils::utils::*;
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::stream_extractor::Thumbnail;
use serde_json::{Map, Value};
use std::convert::TryInto;

#[derive(Clone,PartialEq)]
pub struct YTStreamInfoItemExtractor {
    pub video_info: Map<String, Value>,
}
impl YTStreamInfoItemExtractor {
    pub fn get_name(&self) -> Result<String, ParsingError> {
        if let Some(title) = self.video_info.get("title") {
            let name = get_text_from_object(title, false)?;
            if let Some(name) = name {
                if !name.is_empty() {
                    return Ok(name);
                }
            }
        }
        Err(ParsingError::from("Cannot get name"))
    }

    pub fn is_ad(&self) -> Result<bool, ParsingError> {
        Ok(self.is_premium_video()?
            || self.get_name()? == "[Private video]"
            || self.get_name()? == "[Deleted video]")
    }

    pub fn video_id(&self) -> Result<String, ParsingError> {
        Ok(self
            .video_info
            .get("videoId")
            .ok_or("video id not found")?
            .as_str()
            .ok_or("videoid not string")?
            .to_string())
    }

    pub fn is_premium_video(&self) -> Result<bool, ParsingError> {
        let badges = self
            .video_info
            .get("badges")
            .unwrap_or(&Value::Null)
            .as_array();
        if let Some(badges) = badges {
            for badge in badges {
                if badge
                    .get("metadataBadgeRenderer")
                    .ok_or("metadataBadgeRenderer not found")?
                    .get("label")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("")
                    == "Premium"
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn get_url(&self) -> Result<String, ParsingError> {
        let id = self.video_id()?;
        Ok(format!("https://www.youtube.com/watch?v={}", id))
    }

    pub fn is_live(&self) -> Result<bool, ParsingError> {
        let badges = self
            .video_info
            .get("badges")
            .unwrap_or(&Value::Null)
            .as_array();

        if let Some(badges) = badges {
            for badge in badges {
                if badge
                    .get("metadataBadgeRenderer")
                    .ok_or("metadataBadgeRenderer not found")?
                    .get("label")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("")
                    == "LIVE NOW"
                {
                    return Ok(true);
                }
            }
        }

        let style = self
            .video_info
            .get("thumbnailOverlays")
            .unwrap_or(&Value::Null)
            .get(0)
            .unwrap_or(&Value::Null)
            .get("thumbnailOverlayTimeStatusRenderer")
            .unwrap_or(&Value::Null)
            .get("style")
            .unwrap_or(&Value::Null)
            .as_str()
            .unwrap_or("");
        if style.eq_ignore_ascii_case("LIVE") {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_duration(&self) -> Result<i32, ParsingError> {
        if self.is_live()? {
            return Ok(-1);
        }
        let mut duration = get_text_from_object(
            self.video_info
                .get("lengthText")
                .ok_or("Cant get lengthText")?,
            false,
        )?;
        if duration.is_none() || duration.clone().unwrap_or_default().is_empty() {
            for thumbnail_overlay in self
                .video_info
                .get("thumbnailOverlays")
                .unwrap_or(&Value::Null)
                .as_array()
                .unwrap_or(&vec![])
            {
                if let Some(tr_renderer) =
                    thumbnail_overlay.get("thumbnailOverlayTimeStatusRenderer")
                {
                    duration = get_text_from_object(
                        tr_renderer.get("text").unwrap_or(&Value::Null),
                        false,
                    )?;
                }
            }
        }
        if duration.is_none() || duration.clone().unwrap_or_default().is_empty() {
            Err(ParsingError::from("Cant get duration"))
        } else {
            Ok(remove_non_digit_chars::<i32>(&duration.unwrap_or_default())
                .map_err(|f| ParsingError::from(f.to_string()))?)
        }
    }

    pub fn get_uploader_name(&self) -> Result<String, ParsingError> {
        let mut name = get_text_from_object(
            self.video_info
                .get("longBylineText")
                .unwrap_or(&Value::Null),
            false,
        )?
        .unwrap_or_default();
        if name.is_empty() {
            name = get_text_from_object(
                self.video_info.get("ownerText").unwrap_or(&Value::Null),
                false,
            )?
            .unwrap_or_default();

            if name.is_empty() {
                name = get_text_from_object(
                    self.video_info
                        .get("shortBylineText")
                        .unwrap_or(&Value::Null),
                    false,
                )?
                .unwrap_or_default();

                if name.is_empty() {
                    return Err(ParsingError::from("Cant get uploader name"));
                }
            }
        }

        Ok(name)
    }

    pub fn get_uploader_url(&self) -> Result<String, ParsingError> {
        let mut url = get_url_from_navigation_endpoint(
            self.video_info
                .get("longBylineText")
                .unwrap_or(&Value::Null)
                .get("runs")
                .unwrap_or(&Value::Null)
                .get(0)
                .unwrap_or(&Value::Null)
                .get("navigationEndpoint")
                .unwrap_or(&Value::Null),
        );
        if url.is_err() || url.clone().unwrap_or_default().is_empty() {
            url = get_url_from_navigation_endpoint(
                self.video_info
                    .get("ownerText")
                    .unwrap_or(&Value::Null)
                    .get("runs")
                    .unwrap_or(&Value::Null)
                    .get(0)
                    .unwrap_or(&Value::Null)
                    .get("navigationEndpoint")
                    .unwrap_or(&Value::Null),
            );
            if url.is_err() || url.clone().unwrap_or_default().is_empty() {
                url = get_url_from_navigation_endpoint(
                    self.video_info
                        .get("shortBylineText")
                        .unwrap_or(&Value::Null)
                        .get("runs")
                        .unwrap_or(&Value::Null)
                        .get(0)
                        .unwrap_or(&Value::Null)
                        .get("navigationEndpoint")
                        .unwrap_or(&Value::Null),
                );

                if url.is_err() || url.clone().unwrap_or_default().is_empty() {
                    return Err(ParsingError::from("Cant get uploader url"));
                }
            }
        }
        url
    }

    pub fn get_textual_upload_date(&self) -> Result<String, ParsingError> {
        if self.is_live()? {
            return Err(ParsingError::from("live video has no upload date"));
        }
        let pt = get_text_from_object(
            self.video_info
                .get("publishedTimeText")
                .unwrap_or(&Value::Null),
            false,
        )?;
        Ok(pt.ok_or("Cant get upload date")?)
    }

    pub fn get_view_count(&self) -> Result<i32, ParsingError> {
        if self.is_premium_video()? || self.video_info.contains_key("topStandaloneBadge") {
            return Ok(-1);
        }
        if let Some(viewc) = self.video_info.get("viewCountText") {
            let view_count = get_text_from_object(viewc, false)?.unwrap_or_default();
            if view_count.to_ascii_lowercase().contains("no views") {
                return Ok(0);
            } else if view_count.to_ascii_lowercase().contains("recommended") {
                return Ok(-1);
            } else {
                return Ok(remove_non_digit_chars::<i32>(&view_count)
                    .map_err(|er| ParsingError::from(er.to_string()))?);
            }
        }

        Err(ParsingError::from("Cant get view count"))
    }

    pub fn get_thumbnails(&self) -> Result<Vec<Thumbnail>, ParsingError> {
        let mut thumbnails = vec![];
        for thumb in self
            .video_info
            .get("thumbnail")
            .ok_or("No thumbnail")?
            .get("thumbnails")
            .ok_or("no thumbnails")?
            .as_array()
            .ok_or("thumbnails array")?
        {
            // println!("{:#?}",thumb);
            if let Ok(thumb) = serde_json::from_value(thumb.to_owned()) {
                thumbnails.push(thumb)
            }
        }
        Ok(thumbnails)
    }
}
