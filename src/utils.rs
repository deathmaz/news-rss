use crate::{category::Category, db::DB};
use chrono::DateTime;
use rss::Channel;
use std::io;
use std::process::{Command, Output};

pub fn fetch_feeds(category_list: &Vec<Category>) {
    let db = DB::new();
    for category in category_list {
        for feed_link in category.feed_links() {
            let output = fetch_page(feed_link.to_string());
            match output {
                Ok(result) => {
                    let channel = Channel::read_from(&result.stdout[..]).unwrap();
                    db.create_feed(channel, &feed_link).unwrap();
                }
                Err(e) => println!("Error, {}", e),
            }
        }
    }
}

pub fn fetch_page(url: String) -> io::Result<Output> {
    Command::new("curl").arg("-s").arg("-S").arg(url).output()
}

pub fn formatted_pub_date(date: &str) -> String {
    let parsed = DateTime::parse_from_rfc2822(date).unwrap();
    format!("{}", parsed.format("%d/%m/%Y %H:%M"))
}
