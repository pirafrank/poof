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
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_versions(&mut self, versions: Vec<String>) {
        self.versions = versions;
    }
    pub fn clear_versions(&mut self) {
        self.versions.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
    pub fn contains_version(&self, version: &String) -> bool {
        self.versions.contains(version)
    }
}
