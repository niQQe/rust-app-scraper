use scraper::Html;

pub struct Getter {
    html: String,
    selectors: String,
}

impl Getter {
    pub fn new(url: String, selectors: String) -> Result<Getter, std::io::Error> {
        let html = Getter::fetch_html(url);

        Ok(Getter { html, selectors })
    }
    fn fetch_html(url: String) -> String {
        reqwest::blocking::get(url).unwrap().text().unwrap()
    }
    pub fn get_apps(self) -> Vec<String> {
        let document = Html::parse_document(&self.html);
        let tags = scraper::Selector::parse(&self.selectors).unwrap();
        document
            .select(&tags)
            .map(|x| x.inner_html().trim().to_string())
            .collect::<Vec<String>>()
    }
}
