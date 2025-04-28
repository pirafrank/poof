use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct ReleaseAsset {
    name: String,
    content_type: String,
    //size: u64,
    browser_download_url: String,
}
impl ReleaseAsset {
    pub fn new(name: String, content_type: String, browser_download_url: String) -> Self {
        Self {
            name,
            content_type,
            browser_download_url,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn content_type(&self) -> &String {
        &self.content_type
    }

    pub fn browser_download_url(&self) -> &String {
        &self.browser_download_url
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_content_type(&mut self, content_type: String) {
        self.content_type = content_type;
    }
    pub fn set_browser_download_url(&mut self, browser_download_url: String) {
        self.browser_download_url = browser_download_url;
    }
}
