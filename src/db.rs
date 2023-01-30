use crate::article::Article;
use crate::feed::Feed;
use chrono::DateTime;
use rss::{Channel, Item};
use rusqlite::{Connection, Result};

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new() -> Self {
        let conn =
            Connection::open("news.db").expect("Something went wrong while opening database.");

        Self { conn }
    }

    pub fn create_db(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id              INTEGER PRIMARY KEY,
                title           TEXT,
                rss_link        TEXT NOT NULL UNIQUE,
                link            TEXT NOT NULL,
                description     TEXT,
                pub_date        INTEGER,
                last_build_date INTEGER
            )",
            (),
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                id            INTEGER PRIMARY KEY,
                link          TEXT,
                title         TEXT,
                description   TEXT,
                content       TEXT,
                unread        INTEGER NOT NULL,
                feed_rss_link TEXT NOT NULL,
                guid          TEXT UNIQUE,
                pub_date      INTEGER
            )",
            (),
        )?;
        Ok(())
    }

    fn date_to_timestamp(&self, date: Option<&str>) -> String {
        if let Some(date) = date {
            DateTime::parse_from_rfc2822(date)
                .unwrap()
                .timestamp()
                .to_string()
        } else {
            "".to_string()
        }
    }

    pub fn create_feed(&self, channel: Channel, rss_link: &str) -> Result<()> {
        let pub_date = self.date_to_timestamp(channel.pub_date());

        let last_build_date = self.date_to_timestamp(channel.last_build_date());

        self.conn.execute(
            "INSERT OR IGNORE INTO feeds (
                title       ,
                rss_link    ,
                link        ,
                description ,
                pub_date    ,
                last_build_date
            ) values (
                ?1, ?2, ?3, ?4, ?5, ?6
            )",
            [
                channel.title(),
                rss_link,
                channel.link(),
                channel.description(),
                &pub_date,
                &last_build_date,
            ],
        )?;

        for article in channel.items() {
            self.create_article(article, rss_link)?;
        }
        Ok(())
    }

    pub fn create_article(&self, article: &Item, feed_rss_link: &str) -> Result<()> {
        let guid = match article.guid() {
            Some(guid) => guid.value().to_string(),
            None => String::from(""),
        };
        self.conn.execute(
            "INSERT OR IGNORE INTO articles (
                link      ,
                title     ,
                content   ,
                description,
                unread    ,
                feed_rss_link ,
                guid      ,
                pub_date
            ) values (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
            )",
            [
                article.link().unwrap_or(""),
                article.title().unwrap_or(""),
                article.content().unwrap_or(""),
                article.description().unwrap_or(""),
                &String::from("1"),
                &feed_rss_link,
                &guid,
                &self.date_to_timestamp(article.pub_date()),
            ],
        )?;
        Ok(())
    }

    pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                link      ,
                title     ,
                description,
                content   ,
                unread    ,
                feed_rss_link ,
                guid      ,
                pub_date
            FROM
                articles",
        )?;

        let article_iter = stmt.query_map([], |row| {
            let unread: i8 = row.get(5).unwrap();
            let id: i64 = row.get(0).unwrap();
            let pub_date: Option<i64> = row.get(8).unwrap_or(None);
            Ok(Article::new(
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
                pub_date,
            ))
        })?;
        let mut articles = Vec::new();
        for article in article_iter {
            articles.push(article?);
        }
        Ok(articles)
    }

    pub fn get_articles(&self, rss_link: &str) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                link      ,
                title     ,
                description,
                content   ,
                unread    ,
                feed_rss_link ,
                guid      ,
                pub_date
            FROM
                articles
            WHERE
                feed_rss_link = :rss_link",
        )?;

        let article_iter = stmt.query_map(&[(":rss_link", rss_link)], |row| {
            let unread: i8 = row.get(5).unwrap();
            let id: i64 = row.get(0).unwrap();
            let pub_date: Option<i64> = row.get(8).unwrap_or(None);
            Ok(Article::new(
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
                pub_date,
            ))
        })?;
        let mut articles = Vec::new();
        for article in article_iter {
            articles.push(article?);
        }
        Ok(articles)
    }

    pub fn get_article(&self, article_id: i64) -> Result<Article> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                link      ,
                title     ,
                description,
                content   ,
                unread    ,
                feed_rss_link ,
                guid      ,
                pub_date
            FROM
                articles
            WHERE
                id = :id",
        )?;

        let article = stmt.query_row(&[(":id", &article_id)], |row| {
            let unread: i8 = row.get(5).unwrap();
            let id: i64 = row.get(0).unwrap();
            let pub_date: Option<i64> = row.get(8).unwrap_or(None);
            Ok(Article::new(
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
                pub_date,
            ))
        })?;
        Ok(article)
    }

    pub fn get_feed(&self, rss_link: &str) -> Result<Feed> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                title,
                rss_link,
                link,
                description,
                pub_date,
                last_build_date
            FROM
                feeds
            WHERE
                rss_link = :rss_link",
        )?;
        let feed = stmt.query_row(&[(":rss_link", rss_link)], |row| {
            let pub_date: Option<i64> = row.get(5).unwrap_or(None);
            let last_build_date: Option<i64> = row.get(6).unwrap_or(None);
            let id: i64 = row.get(0).unwrap();
            Ok(Feed::new(
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                pub_date,
                last_build_date,
            ))
        })?;
        Ok(feed)
    }

    pub fn mark_article_as_read(&self, article_id: i64) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "UPDATE
                articles
            SET
                unread = 0
            WHERE
                id = ?",
        )?;
        stmt.execute([article_id])?;
        Ok(())
    }
}
