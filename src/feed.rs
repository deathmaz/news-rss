#[derive(Debug)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub rss_link: String,
    pub feed_id: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<i64>,
    pub category_id: String,
}

impl Feed {
    pub fn new(
        id: i64,
        title: String,
        rss_link: String,
        link: String,
        description: String,
        pub_date: Option<i64>,
        feed_id: String,
        category_id: String,
    ) -> Self {
        Self {
            id,
            title,
            rss_link,
            link,
            description,
            pub_date,
            feed_id,
            category_id,
        }
    }
}
