#[derive(Debug)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub rss_link: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<i64>,
    pub category_id: String,
}

impl Feed {
    pub fn new(
        id: String,
        title: String,
        rss_link: String,
        link: String,
        description: String,
        pub_date: Option<i64>,
        category_id: String,
    ) -> Self {
        Self {
            id,
            title,
            rss_link,
            link,
            description,
            pub_date,
            category_id,
        }
    }
}
