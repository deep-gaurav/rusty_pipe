use crate::utils::utils::remove_non_digit_chars;
use scraper::{ElementRef, Selector};

pub struct YTPlaylistInfoItemExtractor<'a> {
    pub el: ElementRef<'a>,
}

impl YTPlaylistInfoItemExtractor<'_> {
    pub fn get_thumbnail_url(&self) -> Option<&str> {
        let te = self
            .el
            .select(&Selector::parse("div.yt-thumb.video-thumb img").unwrap())
            .next()?;
        let mut url = te.value().attr("src");
        if let Some(url2) = url {
            if url2.contains(".gif") {
                url = te.value().attr("data-thumb");
            }
        }
        url
    }

    pub fn get_name(&self) -> Option<String> {
        let title = self
            .el
            .select(&Selector::parse(".yt-lockup-title a").unwrap())
            .next()?;
        Some(title.text().collect::<Vec<_>>().join(""))
    }

    fn get_premium_url(&self) -> Option<&str> {
        let a = self
            .el
            .select(&Selector::parse("div.yt-lockup-meta  ul.yt-lockup-meta-info li a").unwrap())
            .next()?;
        a.value().attr("href")
    }

    pub fn get_url(&self) -> Option<String> {
        let mut url = self.get_premium_url();
        if url.is_none() {
            url = self
                .el
                .select(&Selector::parse("h3.yt-lockup-title a").unwrap())
                .next()?
                .value()
                .attr("href");
        }
        Some(super::fix_url(url?))
    }

    pub fn get_uploader_name(&self) -> Option<&str> {
        let div = self
            .el
            .select(&Selector::parse("div.yt-lockup-byline a").unwrap())
            .next()?;
        div.text().next()
    }

    pub fn get_stream_count(&self) -> Option<u32> {
        let count = self
            .el
            .select(&Selector::parse("span.formatted-video-count-label b").unwrap())
            .next()?;
        let count_no = remove_non_digit_chars::<u32>(count.text().next()?);
        count_no.ok()
    }
}
