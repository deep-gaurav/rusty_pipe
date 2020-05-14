use crate::utils::utils::remove_non_digit_chars;
use crate::utils::utils::*;
use crate::youtube_extractor::error::ParsingError;
use crate::youtube_extractor::stream_extractor::Thumbnail;
use scraper::{ElementRef, Selector};
use serde_json::{Map, Value};

pub struct YTPlaylistInfoItemExtractor {
    pub playlist_info: Map<String, Value>,
}

impl YTPlaylistInfoItemExtractor {
    pub fn get_thumbnails(&self) -> Result<Vec<Thumbnail>, ParsingError> {
        let mut thumbnails = vec![];
        for thumb in self
            .playlist_info
            .get("thumbnails")
            .ok_or("no thumbnails")?
            .as_array()
            .ok_or("thumbnails array")?
        {
            for thumb in thumb
                .get("thumbnails")
                .ok_or("no nested thumbnails")?
                .as_array()
                .ok_or("thumbnails array")?
            {
                if let Ok(thumb) = serde_json::from_value(thumb.to_owned()) {
                    thumbnails.push(thumb)
                }
            }
        }
        Ok(thumbnails)
    }

    pub fn get_name(&self) -> Result<String, ParsingError> {
        if let Some(title) = self.playlist_info.get("title") {
            let name = get_text_from_object(title, false)?;
            if let Some(name) = name {
                if !name.is_empty() {
                    return Ok(name);
                }
            }
        }
        Err(ParsingError::from("Cannot get name"))
    }

    pub fn playlist_id(&self) -> Result<String, ParsingError> {
        let playlist_id = self
            .playlist_info
            .get("playlistId")
            .ok_or("Cant get playlist id")?
            .as_str()
            .ok_or("Cant get playlist id")?;
        Ok(playlist_id.to_string())
    }

    pub fn get_url(&self) -> Result<String, ParsingError> {
        Ok(format!(
            "https://www.youtube.com/playlist?list={}",
            self.playlist_id()?
        ))
    }

    pub fn get_uploader_name(&self) -> Result<String, ParsingError> {
        match get_text_from_object(
            self.playlist_info
                .get("longBylineText")
                .unwrap_or(&Value::Null),
            false,
        ) {
            Ok(uploader) => Ok(uploader.unwrap_or_default()),
            Err(err) => Err(err),
        }
    }

    pub fn get_stream_count(&self) -> Result<i32, ParsingError> {
        match get_text_from_object(
            self.playlist_info.get("videoCount").unwrap_or(&Value::Null),
            false,
        ) {
            Ok(videos) => {
                // println!("video count {:#?}",videos);
                Ok(remove_non_digit_chars::<i32>(&videos.unwrap_or_default())
                    .map_err(|e| ParsingError::from(e.to_string()))?)
            }
            Err(err) => Err(err),
        }
    }
}
