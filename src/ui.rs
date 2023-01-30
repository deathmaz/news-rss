use crate::article::Article;
use crate::category::Category;
use crate::db::DB;
use crate::tree_entry::TreeEntry;
use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use cursive::{
    traits::*,
    views::{Dialog, LinearLayout, Panel, SelectView},
    Cursive, CursiveRunnable,
};
use ellipse::Ellipse;

use crate::utils;
use cursive_tree_view::{Placement, TreeView};

pub struct UI {
    siv: CursiveRunnable,
}

impl UI {
    pub fn new() -> Self {
        Self {
            siv: cursive::default(),
        }
    }

    pub fn create(&mut self, category_list: Vec<Category>) {
        self.siv.set_user_data(category_list);
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

        self.siv.set_global_callback('R', |siv| {
            let cat_list = siv
                .with_user_data(|user_data: &mut Vec<Category>| user_data.clone())
                .unwrap();
            // TODO: show progress bar
            utils::fetch_feeds(&cat_list);
            // TODO: redraw the panel with tree itrems
            // TODO: redraw the content section
        });

        let mut tree = TreeView::<TreeEntry>::new();

        // FIXME: this element is needed purely to properly align tree elements
        tree.insert_item(
            TreeEntry {
                rss_link: None,
                title: String::from(""),
            },
            Placement::After,
            0,
        );

        tree.set_on_submit(move |siv: &mut Cursive, row| {
            let value = siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
                tree.borrow_item(row).unwrap().clone()
            });
            let v = value.unwrap_or(TreeEntry::default());
            // FIXME: is it ok to create new db connection on each submit?
            // Should it be closed somehow?
            let db = DB::new();
            let articles = db
                .get_articles(&v.rss_link.unwrap_or("".to_string()))
                .unwrap();

            siv.call_on_name("panel", move |view: &mut Dialog| {
                view.set_title(v.title);
            });
            siv.call_on_name("content", move |view: &mut SelectView<Article>| {
                view.clear();
                for article in articles {
                    view.add_item(article.draw(), article);
                }
            });

            siv.focus_name("content").unwrap();
        });

        let db = DB::new();
        let cat_list = self
            .siv
            .with_user_data(|user_data: &mut Vec<Category>| user_data.clone())
            .unwrap();

        for category in cat_list {
            tree.insert_container_item(
                TreeEntry {
                    rss_link: None,
                    title: category.title.to_string(),
                },
                Placement::After,
                0,
            );

            for rss_link in category.feed_links() {
                let feed = db.get_feed(&rss_link).unwrap();
                tree.insert_item(
                    TreeEntry {
                        rss_link: Some(feed.rss_link),
                        title: feed.title,
                    },
                    Placement::LastChild,
                    1,
                );
            }
        }

        // FIXME: hack to properly align elements in tree view
        tree.remove_item(0);

        let mut select = SelectView::<Article>::new();
        select.set_on_submit(|siv: &mut Cursive, item| {
            if item.unread() {
                let db = DB::new();
                db.mark_article_as_read(item.id).unwrap();
                siv.call_on_name("content", move |view: &mut SelectView<Article>| {
                    let id = view.selected_id().unwrap();
                    view.remove_item(id);
                    let article = db.get_article(item.id).unwrap();
                    view.insert_item(id, article.draw(), article.clone());

                    if id == 0 {
                        view.select_up(1);
                    } else {
                        view.select_down(1);
                    }
                });
            }
            siv.add_layer(
                // Dialog::info(selected_id.unwrap().to_string())
                Dialog::around(cursive_markup::MarkupView::html(&item.description).scrollable())
                    .button("Close", |s| {
                        s.pop_layer();
                    })
                    .title(item.title.as_str().truncate_ellipse(70))
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
}
