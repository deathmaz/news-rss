use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct TreeEntry {
    pub title: String,
    pub id: String,
    pub unread_count: Option<i64>,
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unread_count = if let Some(count) = self.unread_count {
            if count == 0 {
                String::from("")
            } else {
                format!("({}) ", count)
            }
        } else {
            String::from("")
        };
        write!(f, "{}{}", unread_count, self.title)
    }
}
