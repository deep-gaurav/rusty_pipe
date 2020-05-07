use scraper::{ElementRef, Selector};

pub struct YTChannelInfoItemExtractor<'a> {
    pub el: ElementRef<'a>,
}
impl YTChannelInfoItemExtractor<'_> {
    pub fn get_thumbnail_url(&self) -> Option<&str> {
        let img = self
            .el
            .select(&Selector::parse("span.yt-thumb-simple img").unwrap())
            .next()?;
        let mut url = img.value().attr("src");
        if let Some(url_s) = url {
            if url_s.contains(".gif") {
                url = img.value().attr("data-thumb");
            }
        }
        url
    }

    pub fn get_name(&self) -> Option<&str> {
        self.el
            .select(&Selector::parse("a.yt-uix-tile-link").unwrap())
            .next()?
            .text()
            .next()
    }

    fn url_using_button(&self) -> Option<String> {
        let button_tracking_url = self
            .el
            .select(&Selector::parse("button[class*=\"yt-uix-button\"]").unwrap())
            .next()?
            .value()
            .attr("data-href")?;
        let channel_id_pattern =
            regex::Regex::new("(?:.*?)%252Fchannel%252F([A-Za-z0-9\\-_]+)(?:.*)").unwrap();
        let capture = channel_id_pattern.captures(button_tracking_url)?;
        Some(super::channel_extractor::CHANNEL_URL_BASE.to_owned() + capture.get(1)?.as_str())
    }

    pub fn get_url(&self) -> Option<String> {
        let url = self.url_using_button();
        if url.is_some() {
            url
        } else {
            if let Some(url) = self
                .el
                .select(&Selector::parse("a[class*=\"yt-uix-tile-link\"]").unwrap())
                .next()?
                .value()
                .attr("href")
            {
                Some(super::fix_url(url))
            } else {
                None
            }
        }
    }

    pub fn get_subscriber_count(&self) -> Option<u32> {
        let subs_el = self
            .el
            .select(&Selector::parse("span[class*=\"yt-subscriber-count\"]").unwrap())
            .next();
        match subs_el {
            None => None,
            Some(subs_el) => {
                let count =
                    super::super::utils::utils::remove_non_digit_chars(subs_el.text().next()?)
                        .expect("Cannot parse to u32");
                Some(count)
            }
        }
    }

    pub fn get_stream_count(&self) -> Option<u32> {
        let meta_el = self
            .el
            .select(&Selector::parse("ul.yt-lockup-meta-info").unwrap())
            .next()?;
        Some(
            super::super::utils::utils::remove_non_digit_chars(meta_el.text().next()?)
                .expect("Cant parse to int"),
        )
    }

    pub fn get_description(&self) -> Option<&str> {
        let des_el = self
            .el
            .select(&Selector::parse("div[class*=\"yt-lockup-description\"]").unwrap())
            .next()?;
        des_el.text().next()
    }
}
