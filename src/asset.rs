//! An installed binary having a repo name and a version is an 'asset'.

pub struct Asset {
    pub name: String,
    pub versions: Vec<String>,
}

#[allow(dead_code)]
impl Asset {
    pub fn new(name: String, versions: Vec<String>) -> Self {
        Self { name, versions }
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_versions(&self) -> &Vec<String> {
        &self.versions
    }
    pub fn add_version(&mut self, version: String) {
        self.versions.push(version);
    }
    pub fn remove_version(&mut self, version: String) {
        self.versions.retain(|v| v != &version);
    }
    pub fn get_latest_version(&self) -> Option<&String> {
        self.versions.last()
    }
}
