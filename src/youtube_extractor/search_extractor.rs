use crate::youtube_extractor::channel_info_item_extractor::YTChannelInfoItemExtractor;
use crate::youtube_extractor::playlist_info_item_extractor::YTPlaylistInfoItemExtractor;
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use scraper::{ElementRef, Html, Selector};
use super::super::downloader_trait::Downloader;

pub enum YTSearchItem<'a> {
    StreamInfoItem(YTStreamInfoItemExtractor<'a>),
    ChannelInfoItem(YTChannelInfoItemExtractor<'a>),
    PlaylistInfoItem(YTPlaylistInfoItemExtractor<'a>),
}

pub struct YTSearchExtractor {
    pub doc: Html,
}

unsafe impl Sync for YTSearchItem<'_>{}

impl YTSearchExtractor {

    pub async fn new<D:Downloader>(downloader:D,query:&str)->Result<YTSearchExtractor,String>{
        let url = format!(
            "https://www.youtube.com/results?disable_polymer=1&search_query={}",
            query
        );
        let resp = downloader.download(&url).await?;
        let doc = Html::parse_document(&resp);

        Ok(
            YTSearchExtractor{
                doc
            }
        )

    }

    pub fn get_search_suggestion(&self) -> Option<String> {
        let el = self
            .doc
            .select(&Selector::parse("div[class*=\"spell-correction\"]").unwrap())
            .next()?;
        let suggestion_el = el.select(&Selector::parse("a").unwrap()).next().unwrap();
        Some(suggestion_el.text().collect::<Vec<_>>().join(""))
    }

    pub fn collect_items(&self) -> Vec<YTSearchItem> {
        let list = self
            .doc
            .select(&Selector::parse("ol[class=\"item-section\"]").unwrap())
            .next()
            .expect("No Item Section");

        let mut search_items: Vec<YTSearchItem> = vec![];

        for item in list.children() {
            let itemel = ElementRef::wrap(item);
            if let Some(itemel) = itemel {
                if itemel
                    .select(&Selector::parse("div[class*=\"search-message\"]").unwrap())
                    .next()
                    .is_some()
                {
                    panic!("Nothing Found")
                }
                if let Some(el) = itemel
                    .select(&Selector::parse("div[class*=\"yt-lockup-video\"").unwrap())
                    .next()
                {
                    search_items.push(YTSearchItem::StreamInfoItem(YTStreamInfoItemExtractor {
                        item: el,
                    }));
                }
                if let Some(el) = itemel
                    .select(&Selector::parse("div[class*=\"yt-lockup-channel\"").unwrap())
                    .next()
                {
                    search_items.push(YTSearchItem::ChannelInfoItem(YTChannelInfoItemExtractor {
                        el,
                    }));
                }
                if let Some(el) = itemel
                    .select(&Selector::parse("div[class*=\"yt-lockup-playlist\"").unwrap())
                    .next()
                {
                    search_items.push(YTSearchItem::PlaylistInfoItem(
                        YTPlaylistInfoItemExtractor { el },
                    ));
                }
            }
        }
        return search_items;
    }

}
