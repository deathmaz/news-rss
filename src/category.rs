#[derive(Debug, Default, Clone)]
pub struct Category {
    pub title: String,
    pub feed_links: Vec<String>,
}

impl Category {
    pub fn new(name: &str, feed_links: Vec<String>) -> Self {
        Self {
            title: name.to_string(),
            feed_links,
        }
    }

    pub fn feed_links(&self) -> Vec<String> {
        self.feed_links.clone()
    }
}
