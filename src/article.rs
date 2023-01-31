use chrono::{Local, TimeZone};

#[derive(Debug, Clone)]
pub struct Article {
    pub id: String,
    pub link: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub unread: i8,
    pub feed_id: String,
    pub pub_date: i64,
}

impl Article {
    pub fn new(
        id: String,
        link: String,
        title: String,
        description: String,
        content: String,
        unread: i8,
        feed_id: String,
        pub_date: i64,
    ) -> Self {
        Self {
            id,
            link,
            title,
            description,
            content,
            unread,
            feed_id,
            pub_date,
        }
    }

    pub fn unread(&self) -> bool {
        self.unread == 1
    }

    pub fn draw(&self) -> String {
        let pub_date = Local.timestamp_opt(self.pub_date, 0).unwrap();
        let unread = if self.unread() { "N" } else { " " };
        format!(
            "{} {} {}",
            pub_date.format("%d/%m/%Y %H:%M"),
            unread,
            self.title,
        )
    }
}
