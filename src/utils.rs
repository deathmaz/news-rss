use crate::{category::Category, db::DB};
use chrono::{Local, TimeZone};
use directories::UserDirs;
use rss::Channel;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

pub fn fetch_feeds(category_list: &Vec<Category>) {
    let db = DB::new();
    for category in category_list {
        for feed_link in category.feed_links() {
            let output = fetch_page(feed_link.to_string());
            match output {
                Ok(result) => {
                    let channel = Channel::read_from(&result.stdout[..]).unwrap();
                    // FIXME: fix this
                    /* db.create_feed(channel, &feed_link, &category.title)
                    .unwrap(); */
                }
                Err(e) => println!("Error, {}", e),
            }
        }
    }
}

pub fn fetch_page(url: String) -> io::Result<Output> {
    Command::new("curl").arg("-s").arg("-S").arg(url).output()
}

pub fn formatted_pub_date(date: i64) -> String {
    let parsed = Local.timestamp_opt(date, 0).unwrap();
    format!("{}", parsed.format("%d/%m/%Y %H:%M"))
}

pub fn get_config_dir() -> String {
    let mut home = String::new();
    if let Some(user_dirs) = UserDirs::new() {
        match user_dirs.home_dir().to_str() {
            Some(path) => home = path.to_string(),
            None => panic!("Can't find home dir!"),
        }
    }
    PathBuf::from(format!("{}/.config/news-rss", home))
        .display()
        .to_string()
}
