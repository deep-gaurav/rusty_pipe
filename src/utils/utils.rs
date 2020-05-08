use serde_json::Value;
use std::collections::hash_map::HashMap;

pub fn remove_non_digit_chars<T: std::str::FromStr>(input: &str) -> Result<T, T::Err> {
    let re = regex::Regex::new("\\D+").unwrap();
    let onlydigits = re.replace_all(input, "");
    let count = onlydigits.parse::<T>();
    count
}

pub fn compat_parse_map(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for arg in input.split("&") {
        let split_arg: Vec<&str> = arg.split("=").collect();
        if let Some(arg_p) = split_arg.get(0) {
            map.insert(
                format!("{}", arg_p),
                String::from(
                    percent_encoding::percent_decode_str(&format!(
                        "{}",
                        split_arg.get(1).unwrap_or(&"")
                    ))
                    .decode_utf8_lossy(),
                ),
            );
        }
    }
    map
}

pub fn fix_thumbnail_url(url: &str) -> String {
    if url.starts_with("//") {
        format!("https:{}", url)
    } else {
        if url.starts_with("http") {
            url.to_string()
        } else {
            format!("https://{}", url)
        }
    }
}

pub fn get_url_from_navigation_endpoint(navigation_endpoint: &Value) -> Result<String, String> {
    if let Some(intern_url) = navigation_endpoint
        .get("urlEndpoint")
        .and_then(|ue| ue.get("url"))
        .and_then(|ue| ue.as_str())
    {
        if intern_url.starts_with("/redirect?") {
            let intern_url = &intern_url[10..];
            for param in intern_url.split("&") {
                if let Some(first_param) = param.split("=").next() {
                    if first_param == "q" {
                        let url = {
                            if let Some(urlencoded) = param.split("=").nth(1) {
                                String::from(
                                    percent_encoding::percent_decode_str(urlencoded)
                                        .decode_utf8_lossy(),
                                )
                            } else {
                                String::new()
                            }
                        };
                        return Ok(url);
                    }
                }
            }
        } else if intern_url.starts_with("http") {
            return Ok(intern_url.to_string());
        }
    } else if let Some(browser_endpoint) = navigation_endpoint.get("browseEndpoint") {
        let canonical_base_url = browser_endpoint
            .get("canonicalBaseUrl")
            .and_then(|c| c.as_str());
        let browse_id = browser_endpoint.get("browseId").and_then(|c| c.as_str());

        if let Some(browse_id) = browse_id {
            if browse_id.starts_with("UC") {
                return Ok(format!("https://www.youtube.com/channel/{}", browse_id));
            }
        }
        if let Some(base_url) = canonical_base_url {
            if !base_url.is_empty() {
                return Ok(format!("https://www.youtube.com{}", base_url));
            }
        }
        return Err("Canonical base url is none, browse id is not channel".to_string());
    } else if let Some(watch_endpoint) = navigation_endpoint.get("watchEndpoint") {
        let mut url = format!(
            "https://www.youtube.com/watch?v={}",
            watch_endpoint
                .get("videoId")
                .and_then(|f| f.as_str())
                .unwrap_or("")
        );
        if let Some(playlist_id) = watch_endpoint.get("playlistId").and_then(|f| f.as_str()) {
            url = url + "&amp;list=" + playlist_id;
        }
        if let Some(start_time_sec) = watch_endpoint
            .get("startTimeSeconds")
            .and_then(|f| f.as_str())
        {
            url = url + "&amp;t=" + start_time_sec;
        }
        return Ok(url);
    } else if let Some(watch_playlist) = navigation_endpoint
        .get("watchPlaylistEndpoint")
        .and_then(|f| f.as_str())
    {
        return Ok(format!(
            "https://www.youtube.com/playlist?list={}",
            watch_playlist
        ));
    }
    Ok("".to_string())
}

pub fn get_text_from_object(text_object: &Value, html: bool) -> Result<Option<String>, String> {
    if let Some(simple_text) = text_object.get("simpleText") {
        return Ok(Some(
            simple_text
                .as_str()
                .ok_or("simple text not string")?
                .to_string(),
        ));
    }
    let mut text = String::new();
    if let Some(runs) = text_object.get("runs").and_then(|runs| runs.as_array()) {
        for text_part in runs {
            let mut text_p = text_part
                .get("text")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string();
            if html {
                if let Some(navp) = text_part.get("navigationEndpoint") {
                    let url = get_url_from_navigation_endpoint(navp)?;
                    if !url.is_empty() {
                        text += &format!("<a href=\"{}\">{}</a>", url, text_p);
                        continue;
                    }
                }
            }
            text += &text_p;
        }
        if html {
            text = text.replace("\n", "<br>");
            text = text.replace(" ", " &nbsp;");
        }
        return Ok(Some(text));
    } else {
        return Ok(None);
    }
}
