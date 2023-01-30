use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct TreeEntry {
    pub title: String,
    pub rss_link: Option<String>,
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}
