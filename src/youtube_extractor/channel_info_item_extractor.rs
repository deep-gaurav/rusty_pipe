use crate::utils::utils::{get_text_from_object, mixed_number_word_parse, remove_non_digit_chars};
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::stream_extractor::Thumbnail;
use scraper::{ElementRef, Selector};
use serde_json::{Map, Value};

#[derive(Clone)]
pub struct YTChannelInfoItemExtractor {
    pub channel_info: Map<String, Value>,
}
impl YTChannelInfoItemExtractor {
    pub fn get_thumbnails(&self) -> Result<Vec<Thumbnail>, ParsingError> {
        let mut thumbnails = vec![];
        for thumb in self
            .channel_info
            .get("thumbnail")
            .ok_or("No thumbnail")?
            .get("thumbnails")
            .ok_or("no thumbnails")?
            .as_array()
            .ok_or("thumbnails array")?
        {
            if let Ok(thumb) = serde_json::from_value(thumb.to_owned()) {
                thumbnails.push(thumb)
            }
        }
        Ok(thumbnails)
    }

    pub fn get_name(&self) -> Result<String, ParsingError> {
        if let Some(title) = self.channel_info.get("title") {
            let name = get_text_from_object(title, false)?;
            if let Some(name) = name {
                if !name.is_empty() {
                    return Ok(name);
                }
            }
        }
        Err(ParsingError::from("Cannot get name"))
    }

    pub fn channel_id(&self) -> Result<String, ParsingError> {
        let channel_id = self
            .channel_info
            .get("channelId")
            .ok_or("Cant get playlist id")?
            .as_str()
            .ok_or("Cant get playlist id")?;
        Ok(channel_id.to_string())
    }

    pub fn get_url(&self) -> Result<String, ParsingError> {
        Ok(format!(
            "https://www.youtube.com/channel/{}",
            self.channel_id()?
        ))
    }

    pub fn get_subscriber_count(&self) -> Result<i32, ParsingError> {
        if let Some(vct) = self.channel_info.get("subscriberCountText") {
            match get_text_from_object(vct, false) {
                Ok(uploader) => Ok(mixed_number_word_parse(&uploader.unwrap_or_default())
                    .map_err(|e| ParsingError::from(e.to_string()))?),
                Err(err) => Err(err),
            }
        } else {
            Ok(-1)
        }
    }

    pub fn get_stream_count(&self) -> Result<i32, ParsingError> {
        if let Some(vct) = self.channel_info.get("videoCountText") {
            match get_text_from_object(vct, false) {
                Ok(uploader) => Ok(remove_non_digit_chars::<i32>(&uploader.unwrap_or_default())
                    .map_err(|e| ParsingError::from(e.to_string()))?),
                Err(err) => Err(err),
            }
        } else {
            Ok(-1)
        }
    }

    pub fn get_description(&self) -> Result<Option<String>, ParsingError> {
        if let Some(vct) = self.channel_info.get("descriptionSnippet") {
            match get_text_from_object(vct, false) {
                Ok(description) => Ok(description),
                Err(err) => Err(err),
            }
        } else {
            Ok(None)
        }
    }
}
