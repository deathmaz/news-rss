use crate::article::Article;
use crate::feed::Feed;
use crate::greader::Category;
use crate::utils;
use rusqlite::{Connection, Result};

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new() -> Self {
        let conn = Connection::open(format!("{}/news.db", utils::get_config_dir()))
            .expect("Something went wrong while opening database.");

        Self { conn }
    }

    pub fn create_db(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            BEGIN;
            CREATE TABLE IF NOT EXISTS unread_articles (
                id  VARCHAR(1024) PRIMARY KEY
            );
             CREATE INDEX IF NOT EXISTS idx_unread_articles_ids ON unread_articles (id);

            CREATE TABLE IF NOT EXISTS categories (
                id              VARCHAR(1024) PRIMARY KEY,
                label           VARCHAR(1024)
            );
            CREATE INDEX IF NOT EXISTS idx_categories_ids ON categories (id);

            CREATE TABLE IF NOT EXISTS feeds (
                id              VARCHAR(1024) PRIMARY KEY,
                title           VARCHAR(1024),
                rss_link        VARCHAR(1024) NOT NULL UNIQUE,
                category_id     VARCHAR(1024) NOT NULL,
                link            VARCHAR(1024) NOT NULL,
                description     VARCHAR(1024),
                pub_date        INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_feeds_ids ON feeds (id);
            CREATE INDEX IF NOT EXISTS idx_feeds_category_ids ON feeds (category_id);

            CREATE TABLE IF NOT EXISTS articles (
                id            VARCHAR(1024) PRIMARY KEY,
                short_id      INTEGER UNIQUE,
                link          VARCHAR(1024),
                title         VARCHAR(1024),
                description   TEXT,
                content       TEXT,
                unread        INTEGER NOT NULL,
                feed_id       VARCHAR(1024) NOT NULL,
                pub_date      INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_articles_ids ON articles (id);
            CREATE INDEX IF NOT EXISTS idx_articles_feed_ids ON articles (feed_id);
            CREATE INDEX IF NOT EXISTS idx_articles_short_ids ON articles (short_id);

            COMMIT;
        ",
        )?;
        Ok(())
    }

    pub fn create_feed(&self, params: CreateFeedParams) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO feeds (
                title       ,
                rss_link    ,
                link        ,
                description ,
                pub_date    ,
                category_id,
                id
            ) values (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7
            )",
            [
                params.title,
                params.rss_link,
                params.link,
                params.description,
                String::from(""),
                params.category_id,
                params.id,
            ],
        )?;
        Ok(())
    }

    pub fn create_category(&self, params: CreateCategoryParams) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO categories (
                id       ,
                label
            ) values (
                ?1, ?2
            )",
            [params.id, params.label],
        )?;
        Ok(())
    }

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                label
            FROM
                categories",
        )?;

        let cat_iter = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                label: row.get(1)?,
            })
        })?;
        let mut categories = Vec::new();
        for category in cat_iter {
            categories.push(category?);
        }
        Ok(categories)
    }

    pub fn get_feed_unread_count(&self, feed_id: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "
                SELECT
                    COUNT(*)
                FROM articles a
                INNER JOIN feeds f ON
                    a.feed_id  = f.id
                WHERE f.id = :feed_id AND a.unread = 1",
        )?;
        let count = stmt.query_row(&[(":feed_id", feed_id)], |row| {
            Ok(UnreadCount { count: row.get(0)? })
        })?;

        Ok(count.count)
    }

    pub fn get_category_unread_count(&self, category_id: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "
                SELECT COUNT(*)
                FROM articles a
                INNER JOIN feeds f ON a.feed_id = f.id
                INNER JOIN categories c ON f.category_id = c.id
                WHERE c.id = :category_id AND a.unread = 1",
        )?;
        let count = stmt.query_row(&[(":category_id", category_id)], |row| {
            Ok(UnreadCount { count: row.get(0)? })
        })?;

        Ok(count.count)
    }

    pub fn create_article(&self, params: CreateArticleParams) -> Result<()> {
        let parts: Vec<&str> = params.id.split("/").collect();
        // See: https://github.com/bazqux/bazqux-api#about-item-ids
        // See: https://github.com/FreshRSS/FreshRSS/blob/edge/p/api/greader.php#L37-L39
        let short_id = i64::from_str_radix(parts.last().unwrap(), 16).unwrap();
        self.conn.execute(
            "INSERT OR IGNORE INTO articles (
                id ,
                short_id ,
                link       ,
                title      ,
                description,
                content    ,
                unread     ,
                feed_id    ,
                pub_date
            ) values (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
            )",
            [
                params.id,
                short_id.to_string(),
                params.link,
                params.title,
                params.description,
                params.content,
                String::from("1"),
                params.feed_id,
                params.pub_date.to_string(),
            ],
        )?;
        Ok(())
    }

    /* pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                link      ,
                title     ,
                description,
                content   ,
                unread    ,
                feed_id ,
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
    } */
    pub fn get_articles_for_feed(&self, feed_id: &str) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                id      ,
                link    ,
                title       ,
                description ,
                content     ,
                unread      ,
                feed_id     ,
                pub_date
            FROM
                articles
            WHERE
                feed_id = :feed_id AND unread = 1
            ORDER BY pub_date DESC",
        )?;

        let article_iter = stmt.query_map(&[(":feed_id", feed_id)], |row| {
            let unread: i8 = row.get(5).unwrap();
            Ok(Article::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
            ))
        })?;
        let mut articles = Vec::new();
        for article in article_iter {
            articles.push(article?);
        }
        Ok(articles)
    }

    pub fn get_articles_for_category(&self, category_id: &str) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                a.id      ,
                a.link    ,
                a.title       ,
                a.description ,
                a.content     ,
                a.unread      ,
                a.feed_id     ,
                a.pub_date
            FROM
                articles a
            INNER JOIN feeds f ON a.feed_id = f.id
            INNER JOIN categories c ON f.category_id = c.id
            WHERE c.id = :category_id AND a.unread = 1
            ORDER BY a.pub_date DESC",
        )?;

        let article_iter = stmt.query_map(&[(":category_id", category_id)], |row| {
            let unread: i8 = row.get(5).unwrap();
            Ok(Article::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
            ))
        })?;
        let mut articles = Vec::new();
        for article in article_iter {
            articles.push(article?);
        }
        Ok(articles)
    }

    /* pub fn get_articles(&self, rss_link: &str) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                link      ,
                title     ,
                description,
                content   ,
                unread    ,
                feed_id ,
                guid      ,
                pub_date
            FROM
                articles
            WHERE
                feed_id = :rss_link",
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
    } */

    pub fn get_article(&self, article_id: String) -> Result<Article> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id      ,
                link    ,
                title       ,
                description ,
                content     ,
                unread      ,
                feed_id     ,
                pub_date
            FROM
                articles
            WHERE
                id = :id",
        )?;

        let article = stmt.query_row(&[(":id", &article_id)], |row| {
            let unread: i8 = row.get(5).unwrap();
            Ok(Article::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                unread,
                row.get(6)?,
                row.get(7)?,
            ))
        })?;
        Ok(article)
    }

    pub fn get_feeds_for_category(&self, category_id: &str) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id ,
                title,
                rss_link,
                link       ,
                description,
                pub_date,
                category_id
            FROM
                feeds
            WHERE
                category_id = :category_id",
        )?;
        let feeds_iter = stmt.query_map(&[(":category_id", category_id)], |row| {
            let pub_date: Option<i64> = row.get(5).unwrap_or(None);
            Ok(Feed::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                pub_date,
                row.get(6)?,
            ))
        })?;
        let mut feeds = Vec::new();
        for feed in feeds_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    /* pub fn get_feed(&self, rss_link: &str) -> Result<Feed> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                id,
                title,
                rss_link,
                link,
                description,
                pub_date,
                category_id
            FROM
                feeds
            WHERE
                rss_link = :rss_link",
        )?;
        let feed = stmt.query_row(&[(":rss_link", rss_link)], |row| {
            let pub_date: Option<i64> = row.get(5).unwrap_or(None);
            let id: i64 = row.get(0).unwrap();
            Ok(Feed::new(
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                pub_date,
                row.get(5)?,
                row.get(6)?,
            ))
        })?;
        Ok(feed)
    } */

    pub fn mark_article_as_read(&self, article_id: &str) -> Result<()> {
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

    pub fn mark_article_as_unread(&self, article_id: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "UPDATE
                articles
            SET
                unread = 1
            WHERE
                id = ?",
        )?;
        stmt.execute([article_id])?;
        Ok(())
    }

    pub fn add_unread_id(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO unread_articles (
                id
            ) values (
                ?1
            )",
            [id],
        )?;
        Ok(())
    }

    pub fn clear_unread_articles(&self) -> Result<()> {
        self.conn.execute("DELETE FROM unread_articles", ())?;
        Ok(())
    }

    pub fn mark_articles_as_read_except(&self, article_ids: Vec<String>) -> Result<()> {
        self.clear_unread_articles()?;
        for id in article_ids {
            self.add_unread_id(&id)?
        }
        let mut stmt = self.conn.prepare(
            "UPDATE
                articles
            SET
                unread = 0
            WHERE unread != 0 AND short_id NOT IN (SELECT id FROM unread_articles)",
        )?;
        stmt.execute([])?;
        Ok(())
    }
}

pub struct CreateFeedParams {
    pub id: String,
    pub title: String,
    pub rss_link: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<String>,
    pub category_id: String,
}

pub struct CreateCategoryParams {
    pub id: String,
    pub label: String,
}

pub struct CreateArticleParams {
    pub id: String,
    pub link: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub unread: i8,
    pub feed_id: String,
    pub pub_date: i64,
}

pub struct UnreadCount {
    count: i64,
}
