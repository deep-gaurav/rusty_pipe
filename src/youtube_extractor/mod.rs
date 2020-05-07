pub mod channel_extractor;
pub mod channel_info_item_extractor;
pub mod itag_item;
pub mod playlist_info_item_extractor;
pub mod search_extractor;
pub mod stream_extractor;
pub mod stream_info_item_extractor;

static YOUTUBE_BASE_URL: &str = "https://www.youtube.com";

fn fix_url(url: &str) -> String {
    if url.starts_with("/") {
        YOUTUBE_BASE_URL.to_owned() + url
    } else {
        url.to_owned()
    }
}
