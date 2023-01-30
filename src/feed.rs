#[derive(Debug)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub rss_link: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<i64>,
    pub last_build_date: Option<i64>,
}

impl Feed {
    pub fn new(
        id: i64,
        title: String,
        rss_link: String,
        link: String,
        description: String,
        pub_date: Option<i64>,
        last_build_date: Option<i64>,
    ) -> Self {
        Self {
            id,
            title,
            rss_link,
            link,
            description,
            pub_date,
            last_build_date,
        }
    }
}
