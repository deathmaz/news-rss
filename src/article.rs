#[derive(Debug, Clone)]
pub struct Article {
    pub id: i64,
    pub link: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub unread: i8,
    pub feed_rss_link: String,
    pub guid: String,
    pub pub_date: Option<i64>,
}

impl Article {
    pub fn new(
        id: i64,
        link: String,
        title: String,
        description: String,
        content: String,
        unread: i8,
        feed_rss_link: String,
        guid: String,
        pub_date: Option<i64>,
    ) -> Self {
        Self {
            id,
            link,
            title,
            description,
            content,
            unread,
            feed_rss_link,
            guid,
            pub_date,
        }
    }

    pub fn unread(&self) -> bool {
        self.unread == 1
    }

    pub fn draw(&self) -> String {
        let unread = if self.unread() { "N" } else { " " };
        format!("{} {} {}", self.pub_date.unwrap_or(0), unread, self.title,)
    }
}
