use scraper::{Html,Selector,ElementRef};
use crate::youtube_extractor::stream_info_item_extractor::YTStreamInfoItemExtractor;
use crate::youtube_extractor::channel_info_item_extractor::YTChannelInfoItemExtractor;
use crate::youtube_extractor::playlist_info_item_extractor::YTPlaylistInfoItemExtractor;


pub enum YTSearchItem<'a>{
    StreamInfoItem(YTStreamInfoItemExtractor<'a>),
    ChannelInfoItem(YTChannelInfoItemExtractor<'a>),
    PlaylistInfoItem(YTPlaylistInfoItemExtractor<'a>)
}


pub struct YTSearchExtractor{
    pub doc:Html
}
impl YTSearchExtractor{

    pub fn get_search_suggestion(&self) ->Option<String>{
        let el = self.doc.select(&Selector::parse("div[class*=\"spell-correction\"]").unwrap()).next()?;
        let suggestion_el=el.select(&Selector::parse("a").unwrap()).next().unwrap();
        Some(suggestion_el.text().collect::<Vec<_>>().join(""))
    }

    pub fn collect_items(&self)->Vec<YTSearchItem>{
        let list = self.doc.select(&Selector::parse("ol[class=\"item-section\"]").unwrap()).next().expect("No Item Section");

        let mut search_items:Vec<YTSearchItem> = vec![];

        for item in list.children(){
            let itemel = ElementRef::wrap(item);
            if let Some(itemel)=itemel{

                if itemel.select(&Selector::parse("div[class*=\"search-message\"]").unwrap()).next().is_some() {
                    panic!("Nothing Found")
                }
                if let Some(el)=itemel.select(&Selector::parse("div[class*=\"yt-lockup-video\"").unwrap()).next(){
                    search_items.push(YTSearchItem::StreamInfoItem(YTStreamInfoItemExtractor{ item: el}));
                }
                if let Some(el)=itemel.select(&Selector::parse("div[class*=\"yt-lockup-channel\"").unwrap()).next(){
                    search_items.push(YTSearchItem::ChannelInfoItem(YTChannelInfoItemExtractor{el}));
                }
                if let Some(el) = itemel.select(&Selector::parse("div[class*=\"yt-lockup-playlist\"").unwrap()).next(){
                    search_items.push(YTSearchItem::PlaylistInfoItem(YTPlaylistInfoItemExtractor{el}));
                }
            }
        }
        return search_items;
    }

}