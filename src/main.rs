use chrono::DateTime;
use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use ellipse::Ellipse;
use rss::{Channel, Item};
use std::fmt;

use std::io;
use std::process::{Command, Output};

use cursive::{
    traits::*,
    views::{Dialog, LinearLayout, Panel, SelectView},
    Cursive, CursiveRunnable,
};

use cursive_tree_view::{Placement, TreeView};

#[derive(Debug, Default, Clone)]
struct Category {
    title: String,
    feed_links: Vec<String>,
    channels: Vec<Channel>,
}

impl Category {
    pub fn new(name: &str, feed_links: Vec<String>) -> Self {
        Self {
            title: name.to_string(),
            feed_links,
            channels: vec![],
        }
    }

    pub fn feed_links(&self) -> Vec<String> {
        self.feed_links.clone()
    }

    pub fn update(&mut self, channel: Channel) {
        self.channels.push(channel);
    }
}

#[derive(Debug, Default, Clone)]
struct TreeEntry {
    title: String,
    description: Option<String>,
    link: Option<String>,
    items: Option<Vec<Item>>,
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

pub fn fetch_page(url: String) -> io::Result<Output> {
    Command::new("curl").arg("-s").arg("-S").arg(url).output()
}

struct App {
    category_list: Vec<Category>,
    siv: CursiveRunnable,
}

impl App {
    pub fn new(category_list: Vec<Category>, siv: CursiveRunnable) -> Self {
        Self { category_list, siv }
    }

    pub fn fetch_feeds(&mut self) {
        for category in self.category_list.iter_mut() {
            for feed_link in category.feed_links() {
                let output = fetch_page(feed_link.to_string());
                match output {
                    Ok(result) => {
                        let channel = Channel::read_from(&result.stdout[..]).unwrap();
                        category.update(channel);
                    }
                    Err(e) => println!("Error, {}", e),
                }
            }
        }
    }

    pub fn create_ui(&mut self) {
        self.siv.set_theme(cursive::theme::Theme {
            shadow: false,
            borders: BorderStyle::Simple,
            palette: Palette::default().with(|palette| {
                use cursive::theme::BaseColor::*;

                {
                    use cursive::theme::Color::TerminalDefault;
                    use cursive::theme::PaletteColor::*;

                    palette[Background] = TerminalDefault;
                    palette[View] = TerminalDefault;
                    palette[Primary] = White.dark();
                    palette[HighlightText] = Black.dark();
                    palette[TitlePrimary] = Blue.light();
                    palette[Secondary] = Blue.light();
                    palette[Highlight] = Blue.dark();
                    palette[HighlightInactive] = Cyan.light();
                }
            }),
        });

        let mut tree = TreeView::<TreeEntry>::new();

        // FIXME: this element is needed purely to properly align tree elements
        tree.insert_item(
            TreeEntry {
                title: String::from(""),
                description: None,
                link: None,
                items: None,
            },
            Placement::After,
            0,
        );

        tree.set_on_submit(move |siv: &mut Cursive, row| {
            let value = siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
                tree.borrow_item(row).unwrap().clone()
            });
            let v = value.unwrap_or(TreeEntry::default());
            match v.items {
                Some(items) => {
                    siv.call_on_name("panel", move |view: &mut Dialog| {
                        view.set_title(v.title);
                    });
                    siv.call_on_name("content", move |view: &mut SelectView<Item>| {
                        view.clear();
                        for item in items {
                            view.add_item(
                                format!(
                                    "{}  {}",
                                    formatted_pub_date(item.pub_date().unwrap_or("")),
                                    item.title().unwrap_or("")
                                ),
                                item.clone(),
                            );
                        }
                    });

                    siv.focus_name("content").unwrap();
                }
                None => (),
            }
        });

        for category in &self.category_list {
            tree.insert_container_item(
                TreeEntry {
                    title: category.title.to_string(),
                    description: None,
                    link: None,
                    items: None,
                },
                Placement::After,
                0,
            );

            for channel in &category.channels {
                tree.insert_item(
                    TreeEntry {
                        title: channel.title().to_string(),
                        description: Some(channel.description.clone()),
                        link: Some(channel.link.clone()),
                        items: Some(channel.items().to_vec()),
                    },
                    Placement::LastChild,
                    1,
                );
            }
        }

        // FIXME: hack to properly align elements in tree view
        tree.remove_item(0);

        let mut select = SelectView::<Item>::new();
        select.set_on_submit(|siv: &mut Cursive, item| {
            siv.add_layer(
                Dialog::around(
                    cursive_markup::MarkupView::html(item.description().unwrap_or("")).scrollable(),
                )
                .button("Close", |s| {
                    s.pop_layer();
                })
                .title(item.title().unwrap_or("").truncate_ellipse(70))
                .max_width(80),
            )
        });

        self.siv.add_fullscreen_layer(
            LinearLayout::horizontal()
                .child(
                    Panel::new(tree.with_name("tree").scrollable())
                        .title("Left sidebar")
                        .with_name("tree_panel")
                        .full_height()
                        .min_width(40),
                )
                .child(
                    Dialog::new()
                        .content(select.with_name("content").scrollable())
                        .title("Content bar")
                        .with_name("panel")
                        .full_height()
                        .full_width(),
                ),
        );

        self.siv.run();
    }

    pub fn run(&mut self) {
        self.fetch_feeds();
        self.create_ui();
    }
}

fn formatted_pub_date(date: &str) -> String {
    let parsed = DateTime::parse_from_rfc2822(date).unwrap();
    format!("{}", parsed.format("%d/%m/%Y %H:%M"))
}

fn main() {
    let category_list = vec![
        Category::new("UaNews", vec![]),
        Category::new("Telegram", vec![]),
    ];
    let siv = cursive::default();
    let mut app = App::new(category_list, siv);
    app.run();
}
