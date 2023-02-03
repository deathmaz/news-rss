use crate::config::Config;
use crate::db::{CreateArticleParams, CreateCategoryParams, CreateFeedParams, DB};
use crate::utils;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Result;
use std::io::Write;
use std::process::Command;

const TOKEN_PREFIX: &str = "Auth";

#[derive(Clone, Debug, Default)]
pub struct Greader {
    cltoken: String,
    api_url: String,
}

impl Greader {
    pub fn login(config: Config) -> Result<Greader> {
        if config.fresh_rss_api_password.is_none()
            || config.fresh_rss_api_user.is_none()
            || config.fresh_rss_api_password.is_none()
        {
            panic!("Some of the FreshRss credentials are missing");
        }

        let output = Command::new("curl")
            .args([
                "-X",
                "POST",
                &format!(
                    "{}/accounts/ClientLogin",
                    config.fresh_rss_api_url.as_ref().unwrap()
                ),
                "-d",
                &format!("Email={}", config.fresh_rss_api_user.unwrap()),
                "-d",
                &format!("Passwd={}", config.fresh_rss_api_password.unwrap()),
            ])
            .output()?;
        let out = String::from_utf8(output.stdout).unwrap();
        let lines = out.lines();
        let mut token = String::from("");
        for item in lines {
            if item.starts_with(TOKEN_PREFIX) {
                let parts: Vec<&str> = item.split("=").collect();
                token = parts[1].trim().to_string();
                break;
            };
        }
        Ok(Greader {
            cltoken: token,
            api_url: config.fresh_rss_api_url.unwrap(),
        })
    }

    pub fn get_unred_articles_content(&self, continuation: Option<String>) -> Result<()> {
        let last_synced = get_last_sync_time();
        let cont = continuation.unwrap_or("".to_string());
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                &format!(
                    "{}/reader/api/0/stream/contents?s=user/-/state/com.google/reading-list&xt=user/-/state/com.google/read&n=1000&r=n&c={}&ot={}",
                    self.api_url,
                    cont,
                    last_synced.trim(),
                ),
            ])
            .output()?;
        let out = String::from_utf8(output.stdout).unwrap();
        let reading_list: ReadingList = serde_json::from_str(&out).unwrap();
        let db = DB::new();
        for item in reading_list.items {
            db.create_article(CreateArticleParams {
                id: item.id,
                link: item.canonical[0].href.clone(),
                title: item.title,
                description: String::from(""),
                content: item.summary.content,
                unread: 1,
                feed_id: item.origin.stream_id,
                pub_date: item.published,
            })
            .unwrap();
        }

        if let Some(con) = reading_list.continuation {
            self.get_unred_articles_content(Some(con)).unwrap();
        } else {
            write_last_sync_time()?;
        }
        Ok(())
    }

    pub fn get_subscription_list(&self) -> Result<()> {
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                &format!(
                    "{}/reader/api/0/subscription/list?output=json",
                    self.api_url
                ),
            ])
            .output()?;
        let out = String::from_utf8(output.stdout).unwrap();
        let subs: Subscriptions = serde_json::from_str(&out).unwrap();
        let db = DB::new();
        for sub in subs.subscriptions {
            let categories = sub.categories;
            for category in &categories {
                // TODO: remove categories that are no longer in the list
                db.create_category(CreateCategoryParams {
                    id: category.id.clone(),
                    label: category.label.clone(),
                })
                .unwrap();
            }
            // TODO: remove feeds that are no longer in the list
            db.create_feed(CreateFeedParams {
                title: sub.title,
                rss_link: sub.url,
                link: sub.html_url,
                description: String::from(""),
                pub_date: None,
                // FIXME: store it as cat_id1,cat_id2,cat_id3?
                category_id: categories[0].id.clone(),
                id: sub.id,
            })
            .unwrap();
        }
        Ok(())
    }

    pub fn get_tag_list(&self) -> Result<()> {
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                &format!("{}/reader/api/0/tag/list?output=json", self.api_url),
            ])
            .output()?;
        let out = String::from_utf8(output.stdout).unwrap();
        let subs: Tags = serde_json::from_str(&out).unwrap();
        for sub in subs.tags {
            println!("{:#?}", sub);
        }
        Ok(())
    }

    pub fn mark_article_as_read(&self, article_id: &str) -> Result<()> {
        Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                "-X",
                "POST",
                &format!("{}/reader/api/0/edit-tag", self.api_url),
                "-d",
                &format!("i={}", article_id),
                "-d",
                "a=user/-/state/com.google/read",
            ])
            .output()?;
        let db = DB::new();
        db.mark_article_as_read(article_id).unwrap();
        Ok(())
    }

    pub fn mark_article_as_unread(&self, article_id: &str) -> Result<()> {
        Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                "-X",
                "POST",
                &format!("{}/reader/api/0/edit-tag", self.api_url),
                "-d",
                &format!("i={}", article_id),
                "-d",
                "r=user/-/state/com.google/read",
            ])
            .output()?;
        let db = DB::new();
        db.mark_article_as_unread(article_id).unwrap();
        Ok(())
    }

    pub fn mark_articles_as_read_except(&self) -> Result<()> {
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization:GoogleLogin auth={}", self.cltoken),
                &format!("{}/reader/api/0/stream/items/ids?output=json&s=user/-/state/com.google/reading-list&xt=user/-/state/com.google/read&n=10000&r=n", self.api_url),
            ])
            .output()?;
        let out = String::from_utf8(output.stdout).unwrap();
        let unread_items: UnreadItemIds = serde_json::from_str(&out).unwrap();
        let mut ids = vec![];
        for item in unread_items.item_refs {
            ids.push(item.id);
        }
        let db = DB::new();
        db.mark_articles_as_read_except(ids).unwrap();
        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        self.get_subscription_list()?;
        self.get_unred_articles_content(None)?;
        self.mark_articles_as_read_except()?;
        Ok(())
    }
}

fn get_last_sync_time() -> String {
    let contents = fs::read_to_string(format!("{}/last_synced", utils::get_config_dir()));
    match contents {
        Ok(t) => t,
        Err(_) => "".to_string(),
    }
}

fn write_last_sync_time() -> Result<()> {
    let mut output = File::create(format!("{}/last_synced", utils::get_config_dir()))?;
    let now = Local::now();
    write!(output, "{}", now.timestamp())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub title: String,
    pub categories: Vec<Category>,
    pub url: String,
    pub html_url: String,
    pub icon_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscriptions {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub id: String,
    pub r#type: Option<String>,
    pub unread_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadingList {
    id: String,
    updated: i64,
    items: Vec<Item>,
    continuation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    id: String,
    published: i64,
    title: String,
    summary: ItemSummary,
    canonical: Vec<ItemCanonical>,
    categories: Vec<String>,
    origin: ItemOrigin,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemSummary {
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemCanonical {
    href: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemOrigin {
    pub stream_id: String,
    pub html_url: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnreadItemId {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnreadItemIds {
    item_refs: Vec<UnreadItemId>,
    continuation: Option<String>,
}
