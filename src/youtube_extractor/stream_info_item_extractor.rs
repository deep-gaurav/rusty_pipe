use scraper::{ElementRef, Selector};
use std::convert::TryInto;

pub struct YTStreamInfoItemExtractor<'a> {
    pub item: ElementRef<'a>,
}
impl YTStreamInfoItemExtractor<'_> {
    pub fn get_name(&self) -> Option<&str> {
        let dl = self.item.select(&Selector::parse("h3").unwrap()).next()?;
        Some(dl.text().next()?)
    }

    pub fn is_ad(&self) -> bool {
        self.item
            .select(&Selector::parse("span[class*=\"icon-not-available\"]").unwrap())
            .next()
            .is_some()
            || self
                .item
                .select(&Selector::parse("span[class*=\"yt-badge-ad\"]").unwrap())
                .next()
                .is_some()
            || self.is_premium_video()
    }

    pub fn is_premium_video(&self) -> bool {
        let premium_span = self
            .item
            .select(
                &Selector::parse("span[class=\"standalone-collection-badge-renderer-red-text\"]")
                    .unwrap(),
            )
            .next();

        match premium_span {
            None => false,
            Some(premium_span) => premium_span.text().next().is_none(),
        }
    }

    pub fn get_url(&self) -> Option<String> {
        let dl = self
            .item
            .select(&Selector::parse("h3").unwrap())
            .next()?
            .select(&Selector::parse("a").unwrap())
            .next()?;
        Some(super::fix_url(dl.value().attr("href")?))
    }

    pub fn is_live(&self) -> bool {
        self.item
            .select(&Selector::parse("span[class*=\"yt-badge-live\"]").unwrap())
            .next()
            .is_some()
            || self
                .item
                .select(&Selector::parse("span[class*=\"video-time-overlay-live\"]").unwrap())
                .next()
                .is_some()
    }

    pub fn get_duration(&self) -> Option<u64> {
        if self.is_live() {
            return None;
        }
        let el = self
            .item
            .select(&Selector::parse("span[class*=\"video-time\"]").unwrap())
            .next()?;
        let duration_text = el.text().next()?;
        let mut splits: Vec<&str> = duration_text.split(":").collect();
        splits.reverse();
        let mut seconds: u64 = 0;
        for i in 0..splits.len() {
            seconds += splits[i].parse::<u64>().unwrap() * 60_u64.pow(i.try_into().unwrap());
        }
        return Some(seconds);
    }

    pub fn get_uploader_name(&self) -> Option<&str> {
        let el = self
            .item
            .select(&Selector::parse("div[class*=\"yt-lockup-byline\"] a").unwrap())
            .next()?;
        //        let el = el.select(&Selector::parse("a").unwrap()).next()?;
        Some(el.text().next()?)
    }

    pub fn get_channel_url(&self) -> Option<String> {
        let el = self
            .item
            .select(&Selector::parse("div[class*=\"yt-lockup-byline\"] a").unwrap())
            .next()?;
        Some(super::fix_url(el.value().attr("href")?))
    }

    pub fn get_uploader_url(&self) -> Option<String> {
        self.get_channel_url()
    }

    pub fn get_textual_upload_date(&self) -> Option<&str> {
        if self.is_live() {
            return None;
        }
        let el = self
            .item
            .select(&Selector::parse("div[class*=\"yt-lockup-meta\"] li").unwrap())
            .next()?;
        el.text().next()
    }

    pub fn get_view_count(&self) -> Option<u32> {
        let input: Option<&str>;
        let span_view_count = self
            .item
            .select(&Selector::parse("span.view-count").unwrap())
            .next();
        if let Some(span_view_count) = span_view_count {
            input = span_view_count.text().next();
        } else if self.is_live() {
            let meta = self
                .item
                .select(&Selector::parse("ul.yt-lockup-meta-info").unwrap())
                .next()?;
            input = meta
                .select(&Selector::parse("li").unwrap())
                .next()?
                .text()
                .next()
        } else {
            let meta = self
                .item
                .select(&Selector::parse("div.yt-lockup-meta").unwrap())
                .next()?;
            let lis = meta
                .select(&Selector::parse("li").unwrap())
                .collect::<Vec<_>>();
            if lis.len() < 2 {
                return None;
            }
            input = lis[1].text().next();
        }

        if let Some(input) = input {
            let count = super::super::utils::utils::remove_non_digit_chars(input)
                .expect("Cannot parse to u32");
            return Some(count);
        } else {
            return None;
        }
    }

    pub fn get_thumbnail_url(&self) -> Option<String> {
        let mut url: Option<&str>;

        let te = self
            .item
            .select(&Selector::parse("div.yt-thumb.video-thumb img").unwrap())
            .next()?;

        url = te.value().attr("src");

        if let Some(url_t) = url {
            if url_t.contains(".gif") {
                url = te.value().attr("data-thumb");
            }
        }
        Some(super::fix_url(url?))
    }
}
