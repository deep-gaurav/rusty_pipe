use super::super::downloader_trait::Downloader;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use super::super::utils::utils::*;
use super::itag_item::{Itag, ItagType};
use std::future::Future;

const CONTENT: &str = "content";

const FORMATS: &str = "formats";
const ADAPTIVE_FORMATS: &str = "adaptiveFormats";
const HTTPS: &str = "https:";
const DECRYPTION_FUNC_NAME: &str = "decrypt";

const VERIFIED_URL_PARAMS: &str = "&has_verified=1&bpctr=9999999999";

const DECRYPTION_SIGNATURE_FUNCTION_REGEX: &str =
    r###"([\w$]+)\s*=\s*function\((\w+)\)\{\s*\2=\s*\2\.split\(""\)\s*;"###;
const DECRYPTION_SIGNATURE_FUNCTION_REGEX_2: &str =
    "\\b([\\w$]{2})\\s*=\\s*function\\((\\w+)\\)\\{\\s*\\2=\\s*\\2\\.split\\(\"\"\\)\\s*;";
const DECRYPTION_AKAMAIZED_STRING_REGEX:&str =
"yt\\.akamaized\\.net/\\)\\s*\\|\\|\\s*.*?\\s*c\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\(";
const DECRYPTION_AKAMAIZED_SHORT_STRING_REGEX: &str =
    "\\bc\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\(";

const HARDCODED_CLIENT_VERSION: &str = "2.20200214.04.00";

pub struct YTStreamExtractor<D: Downloader> {
    doc: String,
    player_args: Map<String, Value>,
    // video_info_page:Map<String,String>,
    player_config: Map<String, Value>,
    player_response: Map<String, Value>,
    player_code: String,
    video_id: String,

    initial_data: Value,
    primary_info_renderer: Value,
    secondary_info_renderer: Value,
    // is_age_restricted:bool,
    downloader: D,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamItem {
    pub url: Option<String>,
    pub itag: u32,
    pub approxDurationMs: String,
    pub audioChannels: Option<u32>,
    pub audioQuality: Option<String>,
    pub audioSampleRate: Option<String>,
    pub averageBitrate: u32,
    pub bitrate: u32,
    pub contentLength: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub quality: String,
    pub qualityLabel: Option<String>,
    pub lastModified: String,
    pub mimeType: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: u64,
    pub height: u64,
}

impl<D: Downloader> YTStreamExtractor<D> {
    pub async fn new(video_id: &str, downloader: D) -> Result<Self, String> {
        use futures::try_join;
        let url = format!(
            "https://www.youtube.com/watch?v={}&disable_polymer=1",
            video_id
        );

        let doc = downloader.download(&url);
        let initial_data = YTStreamExtractor::<D>::get_initial_data(&url, &downloader);
        let (doc, initial_data) = try_join!(doc, initial_data)?;
        if initial_data.1 {
            return Err("age restricted video".to_string());
        }
        let initial_data = initial_data.0;
        let primary_info_renderer =
            YTStreamExtractor::<D>::get_primary_info_renderer(&initial_data)?;
        let secondary_info_renderer =
            YTStreamExtractor::<D>::get_secondary_info_renderer(&initial_data)?;

        let player_config = YTStreamExtractor::<D>::get_player_config(&doc)
            .ok_or("cannot get player_config".to_string())?;
        // println!("player config : {:?}",player_config);

        let player_args = YTStreamExtractor::<D>::get_player_args(&player_config)
            .ok_or("cannot get player args".to_string())?;
        // println!("player args : {:?} ",player_args);

        let player_response = YTStreamExtractor::<D>::get_player_response(&player_args)
            .ok_or("cannot get player response".to_string())?;
        // println!("player response {:?}", player_response);
        let player_url = YTStreamExtractor::<D>::get_player_url(&player_config)
            .ok_or("Cant get player url".to_owned())?;
        let player_code = YTStreamExtractor::<D>::get_player_code(&player_url, &downloader).await?;

        Ok(YTStreamExtractor {
            player_args,
            player_response,
            player_config,
            downloader,
            player_code,
            initial_data,
            primary_info_renderer,
            secondary_info_renderer,
            doc: String::from(doc),
            video_id: String::from(video_id),
        })
    }

    fn get_itags(
        streaming_data_key: &str,
        itag_type_wanted: ItagType,
        player_response: &Map<String, Value>,
        decryption_code: &str,
    ) -> Result<HashMap<String, StreamItem>, String> {
        let mut url_and_itags = HashMap::new();
        let streaming_data = player_response.get("streamingData").unwrap_or(&Value::Null);
        if let Value::Object(streaming_data) = streaming_data {
            if let Value::Array(formats) = streaming_data
                .get(streaming_data_key)
                .unwrap_or(&Value::Null)
            {
                // println!("all formats {:#?}",formats);
                for format_data in formats {
                    if let Value::Object(format_data_obj) = format_data {
                        // println!("format data {:#?}",format_data);
                        let stream_url = match format_data_obj.get("url").unwrap_or(&Value::Null) {
                            Value::String(url) => String::from(url),
                            _ => {
                                let cipherstr = {
                                    if let Value::String(cip) =
                                        format_data_obj.get("cipher").unwrap_or(&Value::Null)
                                    {
                                        cip.clone()
                                    } else {
                                        String::default()
                                    }
                                };
                                let cipher = compat_parse_map(&cipherstr);
                                format!(
                                    "{}&{}={}",
                                    cipher.get("url").unwrap_or(&String::default()),
                                    cipher.get("sp").unwrap_or(&String::default()),
                                    &YTStreamExtractor::<D>::decrypt_signature(
                                        cipher.get("s").unwrap_or(&"".to_owned()),
                                        decryption_code
                                    )
                                )
                            }
                        };
                        match serde_json::from_value::<StreamItem>(format_data.clone()) {
                            Ok(stream_item) => {
                                match itag_type_wanted{
                                    ItagType::VideoOnly=>{
                                        if stream_item.audioQuality.is_none(){
                                            url_and_itags.insert(stream_url, stream_item);
                                        }
                                    }
                                    ItagType::Audio=>{
                                        if stream_item.height.is_none(){
                                            url_and_itags.insert(stream_url, stream_item);
                                        }
                                    }
                                    _=>{
                                        url_and_itags.insert(stream_url, stream_item);
                                    }
                                }
                            }
                            Err(err) => return Err(err.to_string()),
                        }
                    // url_and_itags.insert(stream_url, itag_item);
                    } else {
                        // println!("itag {} rejected",itag);
                    }
                }
            } else {
                return Ok(url_and_itags);
            }
        } else {
            return Err("Streaming data not found in player response".to_string());
        }

        Ok(url_and_itags)
    }

    // pub fn get_video_streams()

    pub async fn get_player_code(player_url: &str, downloader: &D) -> Result<String, String> {
        let player_url = {
            if player_url.starts_with("http://") {
                player_url.to_string()
            } else {
                format!("https://youtube.com{}", player_url)
            }
        };
        let player_code = downloader.download(&player_url).await?;
        let player_code = YTStreamExtractor::<D>::load_decryption_code(&player_code)?;
        Ok(player_code)
    }

    fn decrypt_signature(encrypted_sig: &str, decryption_code: &str) -> String {
        use quick_js::{Context, JsValue};
        let context = Context::new().expect("Cant create js context");
        // println!("decryption code \n{}",decryption_code);
        // println!("signature : {}",encrypted_sig);
        context.eval(decryption_code).expect(&format!(
            "Cant add decryption code to context\n decryption code \n{}",
            decryption_code
        ));
        let result = context.call_function("decrypt", vec![encrypted_sig]);
        // println!("js result : {:?}", result);
        let result = result.expect("Cant exec decrypt");
        let result = result.into_string().expect("Result not string");
        result
    }

    fn get_player_config(page_html: &str) -> Option<Map<String, Value>> {
        let pattern = regex::Regex::new(r"ytplayer.config\s*=\s*(\{.*?\});").ok()?;
        let grp = pattern.captures(page_html)?;
        let yt_player_config_raw = grp.get(1)?.as_str();
        let v: Value = serde_json::from_str(yt_player_config_raw).ok()?;
        if let Value::Object(val) = v {
            return Some(val);
        }
        None
    }

    fn get_player_args(player_config: &Map<String, Value>) -> Option<Map<String, Value>> {
        let args = player_config.get("args")?;
        if let Value::Object(args) = args {
            return Some(args.to_owned());
        }
        None
    }

    fn get_player_url(player_config: &Map<String, Value>) -> Option<String> {
        let yt_assets = player_config.get("assets")?.as_object()?;
        let mut player_url = yt_assets.get("js")?.as_str()?.to_owned();
        if player_url.starts_with("//") {
            player_url = HTTPS.to_owned() + &player_url;
        }
        Some(player_url)
    }

    fn get_player_response(player_args: &Map<String, Value>) -> Option<Map<String, Value>> {
        let player_response_str = player_args.get("player_response")?.as_str()?;
        let player_response: Value = serde_json::from_str(player_response_str).ok()?;
        Some(player_response.as_object()?.to_owned())
    }

    async fn get_initial_data(url: &str, downloader: &D) -> Result<(Value, bool), String> {
        let mut headers = HashMap::new();
        headers.insert("X-YouTube-Client-Name".to_string(), "1".to_string());
        headers.insert(
            "X-YouTube-Client-Version".to_string(),
            HARDCODED_CLIENT_VERSION.to_string(),
        );
        let url = format!("{}&pbj=1", url);
        let data = downloader.download_with_header(&url, headers).await?;
        let initial_ajax_json: Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let initial_ajax_json = initial_ajax_json
            .as_array()
            .ok_or("inital ajax json not array")?;
        if let Some(initial_data) = initial_ajax_json
            .get(2)
            .ok_or("inital ajax 2 not found")?
            .as_object()
        {
            if let Some(response) = initial_data.get("response") {
                Ok((response.clone(), true))
            } else {
                if let Some(initial_data) = initial_ajax_json
                    .get(3)
                    .ok_or("initial ajax 2 not found")?
                    .as_object()
                {
                    if let Some(response) = initial_data.get("response") {
                        Ok((response.clone(), false))
                    } else {
                        Err("Cannot get initial data".to_string())
                    }
                } else {
                    Err("initial ajax doesnt have index 3".to_string())
                }
            }
        } else {
            Err("initial ajax doesnt have index 2".to_string())
        }
        // println!("{:#?}",data)
    }

    fn get_primary_info_renderer(inital_data: &Value) -> Result<Value, String> {
        let contents = inital_data
            .get("contents")
            .and_then(|content| content.get("twoColumnWatchNextResults"))
            .and_then(|content| content.get("results"))
            .and_then(|content| content.get("results"))
            .and_then(|content| content.get("contents"))
            .and_then(|contents| contents.as_array())
            .ok_or("cant get contents")?;

        for content in contents {
            if let Some(info) = content.get("videoPrimaryInfoRenderer") {
                return Ok(info.clone());
            }
        }
        Err("could not get primary info renderer".to_string())
    }
    fn get_secondary_info_renderer(inital_data: &Value) -> Result<Value, String> {
        let contents = inital_data
            .get("contents")
            .and_then(|content| content.get("twoColumnWatchNextResults"))
            .and_then(|content| content.get("results"))
            .and_then(|content| content.get("results"))
            .and_then(|content| content.get("contents"))
            .and_then(|contents| contents.as_array())
            .ok_or("cant get contents")?;

        for content in contents {
            if let Some(info) = content.get("videoSecondaryInfoRenderer") {
                return Ok(info.clone());
            }
        }
        Err("could not get primary info renderer".to_string())
    }

    fn load_decryption_code(player_code: &str) -> Result<String, String> {
        let decryption_func_name = YTStreamExtractor::<D>::get_decryption_func_name(player_code)
            .ok_or("Cant find decryption function")?;

        // println!("Decryption func name {}", decryption_func_name);
        let function_pattern = format!(
            "({}=function\\([a-zA-Z0-9_]+\\)\\{{.+?\\}})",
            decryption_func_name.replace("$", "\\$")
        );

        let decryption_func = format!(
            "var {};",
            YTStreamExtractor::<D>::match_group1(&function_pattern, &player_code)?
        );

        let helper_object_name = YTStreamExtractor::<D>::match_group1(
            ";([A-Za-z0-9_\\$]{2})\\...\\(",
            &decryption_func,
        )?;

        // print!("helper object name : {}",helper_object_name);
        let helper_pattern = format!(
            "(var {}=\\{{.+?\\}}\\}};)",
            helper_object_name.replace("$", "\\$")
        );

        let helper_object =
            YTStreamExtractor::<D>::match_group1(&helper_pattern, &player_code.replace("\n", ""))?;

        let caller_function = format!(
            "function {}(a){{return {}(a);}}",
            DECRYPTION_FUNC_NAME, decryption_func_name
        );

        Ok(format!(
            "{}{}{}",
            helper_object, decryption_func, caller_function
        ))
    }

    fn get_decryption_func_name(player_code: &str) -> Option<String> {
        let decryption_func_name_regexes = vec![
            DECRYPTION_SIGNATURE_FUNCTION_REGEX_2,
            DECRYPTION_SIGNATURE_FUNCTION_REGEX,
            DECRYPTION_AKAMAIZED_SHORT_STRING_REGEX,
            DECRYPTION_AKAMAIZED_STRING_REGEX,
        ];
        for reg in decryption_func_name_regexes {
            let rege = pcre2::bytes::Regex::new(reg).expect("Regex is wrong");
            let capture = rege.captures(player_code.as_bytes()).unwrap();
            if let Some(capture) = capture {
                return capture
                    .get(1)
                    .map(|m| std::str::from_utf8(m.as_bytes()).unwrap().to_owned());
            }
        }
        None
    }

    fn match_group1(reg: &str, text: &str) -> Result<String, String> {
        let rege = pcre2::bytes::Regex::new(reg).expect("Regex is wrong");
        let capture = rege.captures(text.as_bytes()).map_err(|e| e.to_string())?;
        if let Some(capture) = capture {
            return capture
                .get(1)
                .map(|m| std::str::from_utf8(m.as_bytes()).unwrap().to_owned())
                .ok_or("Group 1 not found".to_owned());
        }
        Err("Not matched".to_owned())
    }
}

impl<D: Downloader> YTStreamExtractor<D> {
    pub fn get_name(&self) -> Result<String, String> {
        let mut title = String::new();
        if let Some(title_ob) = self.primary_info_renderer.get("title") {
            let title_ob = get_text_from_object(title_ob, false)?;
            if let Some(title_o) = title_ob {
                title = title_o;
            }
        }
        if title.is_empty() {
            if let Some(t) = self
                .player_response
                .get("videoDetails")
                .and_then(|t| t.get("title"))
                .and_then(|t| t.as_str())
            {
                title = t.to_string();
            }
        }
        if title.is_empty() {
            Err("Cant get title".to_string())
        } else {
            Ok(title)
        }
    }

    pub fn get_description(&self, html: bool) -> Result<(String, bool), String> {
        if let Some(desc) = self.secondary_info_renderer.get("description") {
            let desc = get_text_from_object(desc, html)?;
            if let Some(desc) = desc {
                if !desc.is_empty() {
                    return Ok((desc, true));
                }
            }
        }
        if let Some(desc) = self
            .player_response
            .get("videoDetails")
            .and_then(|f| f.get("shortDescription").and_then(|f| f.as_str()))
        {
            return Ok((desc.to_string(), false));
        }
        Err("Cant get description".to_string())
    }

    pub fn get_video_id(&self) -> String {
        self.video_id.clone()
    }

    pub fn get_video_thumbnails(&self) -> Result<Vec<Thumbnail>, String> {
        if let Value::Object(video_details) = self
            .player_response
            .get("videoDetails")
            .ok_or("cant get video Details")?
        {
            if let Value::Object(thumbnail) =
                video_details.get("thumbnail").ok_or("cant get thumbnail")?
            {
                if let Value::Array(thumbnails) = thumbnail
                    .get("thumbnails")
                    .ok_or("Cant get thumbnails array")?
                {
                    let mut thumbnails_str = vec![];
                    for thumb in thumbnails {
                        let mut thumbnail: Thumbnail =
                            serde_json::from_value(thumb.clone()).map_err(|e| e.to_string())?;
                        thumbnail.url = fix_thumbnail_url(&thumbnail.url);
                        thumbnails_str.push(thumbnail)
                    }
                    return Ok(thumbnails_str);
                }
            }
        }
        Err("Cant get video thumbnails".to_string())
    }

    pub fn get_length(&self) -> Result<u64, String> {
        if let Some(duration) = self
            .player_response
            .get("videoDetails")
            .and_then(|f| f.get("lengthSeconds"))
            .and_then(|f| f.as_str())
        {
            if let Ok(duration) = duration.parse::<u64>() {
                return Ok(duration);
            }
        }
        if let Some(duration_ms) = self
            .player_response
            .get("streamingData")
            .and_then(|f| f.get("formats"))
            .and_then(|f| f.as_array())
            .and_then(|f| f.get(0))
            .and_then(|f| f.get("approxDurationMs"))
            .and_then(|f| f.as_str())
        {
            if let Ok(duration) = duration_ms.parse::<u64>() {
                return Ok(duration / 1000);
            }
        }

        Err("Cant get duration".to_string())
    }

    pub fn get_view_count(&self) -> Result<u128, String> {
        let mut views = String::new();
        if let Some(vc) = self
            .primary_info_renderer
            .get("viewCount")
            .and_then(|f| f.get("videoViewCountRenderer"))
            .and_then(|f| f.get("viewCount"))
        {
            views = get_text_from_object(vc, false)?.unwrap_or("".to_string());
        }
        if views.is_empty() {
            if let Some(vc) = self
                .player_response
                .get("videoDetails")
                .and_then(|f| f.get("viewCount"))
                .and_then(|f| f.as_str())
            {
                views = vc.to_string();
            }
        }
        if !views.is_empty() {
            if views.to_ascii_lowercase().contains("no views") {
                return Ok(0);
            } else {
                if let Ok(views) = remove_non_digit_chars::<u128>(&views) {
                    return Ok(views);
                }
            }
        }
        // println!("{}",views);
        Err("Cant get view count".to_string())
    }

    pub fn get_like_count(&self) -> Result<i128, String> {
        let mut like_string = String::new();
        if let Some(likes) = self
            .primary_info_renderer
            .get("sentimentBar")
            .and_then(|f| f.get("sentimentBarRenderer"))
            .and_then(|f| f.get("tooltip"))
            .and_then(|f| f.as_str())
        {
            if let Some(lks) = likes.split("/").next() {
                like_string = lks.to_string();
            }
        }
        if like_string.is_empty() {
            if let Some(allow_ratings) = self
                .player_response
                .get("videoDetails")
                .and_then(|f| f.get("allowRatings"))
                .and_then(|f| f.as_bool())
            {
                if allow_ratings {
                    return Err(
                        "Ratings are enabled even though the like button is missing".to_string()
                    );
                } else {
                    return Ok(-1);
                }
            }
        } else {
            if let Ok(likes) = remove_non_digit_chars::<i128>(&like_string) {
                return Ok(likes);
            }
        }
        Err("could not get like count".to_string())
    }

    pub fn get_dislike_count(&self) -> Result<i128, String> {
        let mut like_string = String::new();
        if let Some(likes) = self
            .primary_info_renderer
            .get("sentimentBar")
            .and_then(|f| f.get("sentimentBarRenderer"))
            .and_then(|f| f.get("tooltip"))
            .and_then(|f| f.as_str())
        {
            if let Some(lks) = likes.split("/").nth(1) {
                like_string = lks.to_string();
            }
        }
        if like_string.is_empty() {
            if let Some(allow_ratings) = self
                .player_response
                .get("videoDetails")
                .and_then(|f| f.get("allowRatings"))
                .and_then(|f| f.as_bool())
            {
                if allow_ratings {
                    return Err(
                        "Ratings are enabled even though the dislike button is missing".to_string(),
                    );
                } else {
                    return Ok(-1);
                }
            }
        } else {
            if let Ok(likes) = remove_non_digit_chars::<i128>(&like_string) {
                return Ok(likes);
            }
        }
        Err("could not get dislike count".to_string())
    }

    pub fn get_uploader_url(&self) -> Result<String, String> {
        if let Some(nav_end) = self
            .secondary_info_renderer
            .get("owner")
            .and_then(|f| f.get("videoOwnerRenderer"))
            .and_then(|f| f.get("navigationEndpoint"))
        {
            let uploader_url = get_url_from_navigation_endpoint(nav_end)?;
            if !uploader_url.is_empty() {
                return Ok(uploader_url);
            }
        }
        if let Some(uploader_id) = self
            .player_response
            .get("videoDetails")
            .and_then(|f| f.get("channelId"))
            .and_then(|f| f.as_str())
        {
            return Ok(format!("https://www.youtube.com/channel/{}", uploader_id));
        }
        Err("Cant get uploader url".to_string())
    }

    pub fn get_uploader_name(&self) -> Result<String, String> {
        let mut uploader_name = String::new();
        if let Some(uploader) = self
            .secondary_info_renderer
            .get("owner")
            .and_then(|f| f.get("videoOwnerRenderer"))
            .and_then(|f| f.get("title"))
        {
            if let Some(uploader) = get_text_from_object(uploader, false)? {
                uploader_name = uploader;
            }
        }
        if uploader_name.is_empty() {
            if let Some(author) = self
                .player_response
                .get("videoDetails")
                .and_then(|f| f.get("author"))
                .and_then(|f| f.as_str())
            {
                uploader_name = author.to_owned();
            }
        }

        if uploader_name.is_empty() {
            Err("Cant get uploader name".to_string())
        } else {
            Ok(uploader_name)
        }
    }

    pub fn get_uploader_avatar_url(&self) -> Result<Vec<Thumbnail>, String> {
        let mut thumbnails = vec![];
        if let Some(thumbs) = self
            .secondary_info_renderer
            .get("owner")
            .and_then(|f| f.get("videoOwnerRenderer"))
            .and_then(|f| f.get("thumbnail"))
            .and_then(|f| f.get("thumbnails"))
            .and_then(|f| f.as_array())
        {
            for thumb in thumbs {
                if let Ok(mut thumb) = serde_json::from_value::<Thumbnail>(thumb.clone()) {
                    thumb.url = fix_thumbnail_url(&thumb.url);
                    thumbnails.push(thumb);
                }
            }
        }
        Ok(thumbnails)
    }

    // pub fn is_live(&self)->Result<bool,String>{
    //     if let Some(format) = self.player_response.get("streamingData").and_then(|f|f.get(FORMATS)){
    //         return Ok(true);
    //     }else if let Some(ps)= self.player_args.get("ps").and_then(|f|f.as_str()){
    //         println!("{}",ps);
    //         if ps=="live"{
    //             return Ok(true)
    //         }
    //     }
    //     Ok(false)
    // }
}

impl<D: Downloader> YTStreamExtractor<D> {
    pub fn get_video_stream(&self) -> Result<Vec<StreamItem>, String> {
        let mut video_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(
            FORMATS,
            ItagType::Video,
            &self.player_response,
            &self.player_code,
        )? {
            let itag = entry.1;
            video_streams.push(StreamItem {
                url: Some(entry.0),
                ..itag
            });
        }
        Ok(video_streams)
    }

    pub fn get_video_only_stream(&self) -> Result<Vec<StreamItem>, String> {
        let mut video_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(
            ADAPTIVE_FORMATS,
            ItagType::VideoOnly,
            &self.player_response,
            &self.player_code,
        )? {
            let itag = entry.1;
            video_streams.push(StreamItem {
                url: Some(entry.0),
                ..itag
            });
        }
        Ok(video_streams)
    }

    pub fn get_audio_streams(&self) -> Result<Vec<StreamItem>, String> {
        let mut audio_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(
            ADAPTIVE_FORMATS,
            ItagType::Audio,
            &self.player_response,
            &self.player_code,
        )? {
            let itag = entry.1;
            audio_streams.push(StreamItem {
                url: Some(entry.0),
                ..itag
            });
        }

        Ok(audio_streams)
    }
}
