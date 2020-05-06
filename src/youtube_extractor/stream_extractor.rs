use scraper::{Html, Selector};
use serde_json::{Value, Map};
use std::collections::HashMap;
use super::super::downloader_trait::Downloader;

use super::itag_item::{ItagType,Itag};
use super::super::utils::utils::*;
use std::future::Future;

const CONTENT:&str = "content";

const FORMATS:&str = "formats";
const ADAPTIVE_FORMATS:&str = "adaptiveFormats";
const HTTPS:&str = "https:";
const DECRYPTION_FUNC_NAME:&str = "decrypt";

const VERIFIED_URL_PARAMS:&str = "&has_verified=1&bpctr=9999999999";

const DECRYPTION_SIGNATURE_FUNCTION_REGEX:&str =
r###"([\w$]+)\s*=\s*function\((\w+)\)\{\s*\2=\s*\2\.split\(""\)\s*;"###;
const DECRYPTION_SIGNATURE_FUNCTION_REGEX_2:&str =
"\\b([\\w$]{2})\\s*=\\s*function\\((\\w+)\\)\\{\\s*\\2=\\s*\\2\\.split\\(\"\"\\)\\s*;";
const DECRYPTION_AKAMAIZED_STRING_REGEX:&str =
"yt\\.akamaized\\.net/\\)\\s*\\|\\|\\s*.*?\\s*c\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\(";
const DECRYPTION_AKAMAIZED_SHORT_STRING_REGEX:&str =
"\\bc\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\(";

pub struct YTStreamExtractor<D:Downloader>{
    doc:String,
    player_args:Map<String,Value>,
    // video_info_page:Map<String,String>,
    player_config:Map<String,Value>,
    player_response:Map<String,Value>,
    player_code:String,

    // is_age_restricted:bool,
    downloader: D
}

#[derive(Debug)]
pub struct StreamItem{
    pub url:String,
    pub itag:Itag
}

impl<D:Downloader> YTStreamExtractor<D>{

    fn get_name(&self)->Option<String>{

        let vid =self.player_response.get("videoDetails");
        if let Some(vid) = vid{
            if let Value::Object(vid) = vid{
                if let Value::String(title)=vid.get("title")?{
                    return Some(title.to_owned())
                };
            }
        }

        let doc_html = Html::parse_document(&self.doc);

        let title = doc_html.select(&Selector::parse("meta[name=title]").unwrap())
            .next()?.value().attr(CONTENT)?;
        Some(title.to_owned())
    
    }

    pub async fn new(doc:&str,_url:&str, downloader: D) -> Result<Self,String> {
        let player_config = YTStreamExtractor::<D>::get_player_config(doc).ok_or("cannot get player_config".to_string())?;
        // println!("player config : {:?}",player_config);

        let player_args = YTStreamExtractor::<D>::get_player_args(&player_config).ok_or("cannot get player args".to_string())?;
        // println!("player args : {:?} ",player_args);

        let player_response = YTStreamExtractor::<D>::get_player_response(&player_args).ok_or("cannot get player response".to_string())?;
        // println!("player response {:?}", player_response);
        let player_url = YTStreamExtractor::<D>::get_player_url(&player_config).ok_or("Cant get player url".to_owned())?;
        let player_code = YTStreamExtractor::<D>::get_player_code(&player_url,&downloader).await?;

        Ok(
            YTStreamExtractor{
                player_args,
                player_response,
                player_config,
                downloader,
                player_code,
                doc:String::from(doc)
            }
        )

    }

    fn get_itags(streaming_data_key:&str, itag_type_wanted:ItagType, player_response:&Map<String,Value>,decryption_code:&str)-> Result<HashMap<String,Itag>,String>{
        let mut url_and_itags = HashMap::new();
        let streaming_data = player_response.get("streamingData").unwrap_or(&Value::Null);
        if let Value::Object(streaming_data) = streaming_data{
            if let Value::Array(formats) = streaming_data.get(streaming_data_key).unwrap_or(&Value::Null){
                // println!("all formats {:#?}",formats);
                for format_data in formats{
                    if let Value::Object(format_data) = format_data{
                        if let Value::Number(itag) = format_data.get("itag").unwrap_or(&Value::Null){
                            // println!("check itag {}",itag);
                            if let Ok(itag_item) = Itag::get_itag(itag.as_i64().unwrap_or_default()){
                                if itag_item.itag_type == itag_type_wanted{
                                    // println!("itag {} accepted",itag);
                                    let stream_url = match format_data.get("url").unwrap_or(&Value::Null){
                                        Value::String(url)=> String::from(url),
                                        _ => {
                                            let cipherstr = {
                                                if let Value::String(cip)= format_data.get("cipher").unwrap_or(&Value::Null){
                                                    cip.clone()
                                                }else{
                                                    String::default()
                                                }
                                            };
                                            let cipher = compat_parse_map(&cipherstr);
                                            format!("{}&{}={}",
                                                cipher.get("url").unwrap_or(&String::default()),
                                                cipher.get("sp").unwrap_or(&String::default()),
                                                &YTStreamExtractor::<D>::decrypt_signature(cipher.get("s").unwrap_or(&"".to_owned()), decryption_code)
                                            )
                                        }
                                    };
                                    url_and_itags.insert(stream_url, itag_item);
                                }else{
                                    // println!("itag {} rejected",itag);
                                }
                            }
                        }
                    }
                }
            }else{
                return Ok(url_and_itags);
            }
        }else{
            return Err("Streaming data not found in player response".to_string());
        }

        Ok(url_and_itags)
    }

    // pub fn get_video_streams()

    pub async fn get_player_code(player_url:&str,downloader:&D)->Result<String,String>{
        let player_url = {
            if player_url.starts_with("http://"){
                player_url.to_string()
            }else{
                format!("https://youtube.com{}",player_url)
            }
        };
        let player_code = downloader.download(&player_url).await?;
        let player_code = YTStreamExtractor::<D>::load_decryption_code(&player_code)?;
        Ok(player_code)

    }

    pub async fn get_video_stream(&mut self)->Result<Vec<StreamItem>,String>{

        let mut video_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(FORMATS,ItagType::Video,&self.player_response,&self.player_code)?{
            let itag = entry.1;
            video_streams.push(
                StreamItem{
                    url:entry.0,
                    itag
                }
            );
        }
        Ok(video_streams)
    }

    pub async fn get_video_only_stream(&mut self)->Result<Vec<StreamItem>,String>{

        let mut video_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(ADAPTIVE_FORMATS,ItagType::VideoOnly,&self.player_response,&self.player_code)?{
            let itag = entry.1;
            video_streams.push(
                StreamItem{
                    url:entry.0,
                    itag
                }
            );
        }
        Ok(video_streams)
    }

    pub async fn get_audio_streams(&mut self)->Result<Vec<StreamItem>,String>{
        let mut audio_streams = vec![];
        for entry in YTStreamExtractor::<D>::get_itags(ADAPTIVE_FORMATS,ItagType::Audio,&self.player_response,&self.player_code)?{
            let itag = entry.1;
            audio_streams.push(
                StreamItem{
                    url:entry.0,
                    itag
                }
            );
        }

        Ok(audio_streams)
    }

    fn decrypt_signature(encrypted_sig:&str,decryption_code:&str)->String{
        use quick_js::{Context,JsValue};
        let context = Context::new().expect("Cant create js context");
        // println!("decryption code \n{}",decryption_code);
        // println!("signature : {}",encrypted_sig);
        context.eval(decryption_code).expect(&format!("Cant add decryption code to context\n decryption code \n{}",decryption_code));
        let result = context.call_function("decrypt", vec![encrypted_sig]);
        // println!("js result : {:?}", result);
        let result = result.expect("Cant exec decrypt");
        let result = result.into_string().expect("Result not string");
        result
    }

    fn get_player_config(page_html:&str)->Option<Map<String,Value>>{
        let pattern = regex::Regex::new(r"ytplayer.config\s*=\s*(\{.*?\});").ok()?;
        let grp = pattern.captures(page_html)?;
        let yt_player_config_raw = grp.get(1)?.as_str();
        let v:Value = serde_json::from_str(yt_player_config_raw).ok()?;
        if let Value::Object(val) = v{
            return Some(val)
        }
        None
    }

    fn get_player_args(player_config:&Map<String,Value>)->Option<Map<String,Value>>{

        let args = player_config.get("args")?;
        if let Value::Object(args)= args{
            return Some(args.to_owned())
        }
        None

    }

    fn get_player_url(player_config:&Map<String,Value>)->Option<String>{
        let yt_assets = player_config.get("assets")?.as_object()?;
        let mut player_url = yt_assets.get("js")?.as_str()?.to_owned();
        if player_url.starts_with("//"){
            player_url = HTTPS.to_owned()+&player_url;
        }
        Some(player_url)
    }

    fn get_player_response(player_args:&Map<String,Value>) -> Option<Map<String,Value>>{
        let player_response_str = player_args.get("player_response")?.as_str()?;
        let player_response:Value = serde_json::from_str(player_response_str).ok()?;
        Some(player_response.as_object()?.to_owned())
    }



    fn load_decryption_code(player_code:&str)->Result<String,String>{
        let decryption_func_name = YTStreamExtractor::<D>::get_decryption_func_name(player_code).ok_or("Cant find decryption function")?;

        // println!("Decryption func name {}", decryption_func_name);
        let function_pattern = format!(
            "({}=function\\([a-zA-Z0-9_]+\\)\\{{.+?\\}})",
            decryption_func_name.replace("$", "\\$")
        );

        let decryption_func = format!(
            "var {};",
            YTStreamExtractor::<D>::match_group1(&function_pattern, &player_code)?
        );

        let helper_object_name = YTStreamExtractor::<D>::match_group1(";([A-Za-z0-9_\\$]{2})\\...\\(", &decryption_func)?;

        // print!("helper object name : {}",helper_object_name);
        let helper_pattern = format!(
            "(var {}=\\{{.+?\\}}\\}};)",
            helper_object_name.replace("$", "\\$")
        );

        let helper_object = YTStreamExtractor::<D>::match_group1(&helper_pattern, &player_code.replace("\n", ""))?;

        let caller_function = format!(
            "function {}(a){{return {}(a);}}",
            DECRYPTION_FUNC_NAME,
            decryption_func_name
        );

        Ok(format!("{}{}{}",helper_object,decryption_func,caller_function))
    }

    fn get_decryption_func_name(player_code:&str)->Option<String>{
        let decryption_func_name_regexes = vec![
            DECRYPTION_SIGNATURE_FUNCTION_REGEX_2,
            DECRYPTION_SIGNATURE_FUNCTION_REGEX,
            DECRYPTION_AKAMAIZED_SHORT_STRING_REGEX,
            DECRYPTION_AKAMAIZED_STRING_REGEX
        ];
        for reg in decryption_func_name_regexes{
            let rege = pcre2::bytes::Regex::new(reg).expect("Regex is wrong");
            let capture = rege.captures(player_code.as_bytes()).unwrap();
            if let Some(capture) = capture{
                return capture.get(1).map(|m|std::str::from_utf8(m.as_bytes()).unwrap().to_owned());
            }
        }
        None
    }

    fn match_group1(reg:&str,text:&str)->Result<String,String>{
        let rege = pcre2::bytes::Regex::new(reg).expect("Regex is wrong");
        let capture = rege.captures(text.as_bytes()).map_err(|e|e.to_string())?;
        if let Some(capture) = capture{
            return capture.get(1).map(|m|std::str::from_utf8(m.as_bytes()).unwrap().to_owned()).ok_or("Group 1 not found".to_owned());
        }
        Err("Not matched".to_owned())
    }
}
