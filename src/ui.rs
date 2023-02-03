use crate::article::Article;
use crate::db::DB;
use crate::greader::Category;
use crate::greader::Greader;
use crate::tree_entry::TreeEntry;
use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use cursive::views::OnEventView;
use cursive::{
    traits::*,
    views::{Dialog, LinearLayout, Panel, SelectView},
    Cursive, CursiveRunnable,
};
use ellipse::Ellipse;

use cursive_tree_view::{Placement, TreeView};

pub struct UI {
    siv: CursiveRunnable,
}

#[derive(Clone)]
struct UserData {
    category_list: Vec<Category>,
    greader: Greader,
}

impl UI {
    pub fn new() -> Self {
        Self {
            siv: cursive::default(),
        }
    }

    pub fn create(&mut self, greader: Greader) {
        let db = DB::new();
        let category_list = db.get_categories().unwrap();
        self.siv.set_user_data(UserData {
            category_list,
            greader,
        });

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
        tree.set_on_collapse(tree_on_collapse);
        tree.set_on_submit(move |siv: &mut Cursive, row| {
            let db = DB::new();
            let value = siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
                tree.borrow_item(row).unwrap().clone()
            });
            if let Some(v) = value {
                let articles = db.get_articles_for_feed(&v.id).unwrap();

                // FIXME: Find a way how to update feed unread count when the article was read from
                // focused category
                let len = articles
                    .iter()
                    .filter(|a| a.unread())
                    .collect::<Vec<_>>()
                    .len();

                siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
                    let item = tree.borrow_item_mut(row).unwrap();
                    item.unread_count = Some(len.try_into().unwrap());
                });

                draw_articles(articles, siv, &v.title);
            }
        });

        let cat_list = self
            .siv
            .with_user_data(|user_data: &mut UserData| user_data.category_list.clone())
            .unwrap();
        build_tree(cat_list, &mut tree);

        self.siv.set_global_callback('R', move |siv| {
            let user_data = siv
                .with_user_data(|user_data: &mut UserData| user_data.clone())
                .unwrap();

            user_data.greader.sync().unwrap();

            siv.call_on_name("tree", |tree: &mut TreeView<TreeEntry>| {
                /* let selected_row = tree.row().unwrap();
                let item = tree.borrow_item_mut(selected_row).unwrap();
                item.unread_count = Some(0); */
                tree.clear();
                build_tree(user_data.category_list, tree);
                // tree.set_collapsed(selected_row, false);
                // tree.set_selected_row(selected_row);
            });
            siv.focus_name("tree").unwrap();
            // siv.pop_layer();
            // TODO: show progress bar
            // TODO: redraw the panel with tree itrems
            // TODO: redraw the content section
        });

        let mut select = SelectView::<Article>::new();
        select.set_on_submit(|siv: &mut Cursive, item| {
            if item.unread() {
                let db = DB::new();
                let greader = siv
                    .with_user_data(|user_data: &mut UserData| user_data.greader.clone())
                    .unwrap();
                greader.mark_article_as_read(&item.id).unwrap();

                siv.call_on_name("content", move |view: &mut SelectView<Article>| {
                    let id = view.selected_id().unwrap();
                    view.remove_item(id);
                    let article = db.get_article(item.id.clone()).unwrap();
                    view.insert_item(id, article.draw(), article.clone());

                    if id == 0 {
                        view.select_up(1);
                    } else {
                        view.select_down(1);
                    }
                });

                siv.call_on_name("tree", |tree: &mut TreeView<TreeEntry>| {
                    let selected_row = tree.row().unwrap();
                    decrease_unread_count(tree, selected_row);
                });

                siv.call_on_name("tree", |tree: &mut TreeView<TreeEntry>| {
                    let selected_row = tree.row().unwrap();
                    let parent = tree.item_parent(selected_row);
                    if let Some(p) = parent {
                        decrease_unread_count(tree, p);
                    }
                });
            }

            siv.add_layer(
                // Dialog::info(selected_id.unwrap().to_string())
                Dialog::around(cursive_markup::MarkupView::html(&item.content).scrollable())
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
                        .max_width(40)
                        .min_width(20),
                )
                .child(
                    Dialog::new()
                        .content(
                            OnEventView::new(select.with_name("content").scrollable())
                                .on_event('j', content_select_down)
                                .on_event('k', content_select_up),
                        )
                        .title("Content bar")
                        .with_name("panel")
                        .full_height()
                        .full_width(),
                ),
        );

        self.siv.run();
    }
}

fn build_tree(cat_list: Vec<Category>, tree: &mut TreeView<TreeEntry>) {
    let db = DB::new();
    // FIXME: this element is needed purely to properly align tree elements
    tree.insert_item(
        TreeEntry {
            id: String::from("dummy"),
            title: String::from(""),
            unread_count: None,
        },
        Placement::After,
        0,
    );

    for category in cat_list {
        let unread_count = db.get_category_unread_count(&category.id).unwrap();
        tree.insert_container_item(
            TreeEntry {
                id: category.id.clone(),
                title: category.label.to_string(),
                unread_count: Some(unread_count),
            },
            Placement::After,
            0,
        );
        let feeds = db.get_feeds_for_category(&category.id).unwrap();
        for feed in feeds {
            let unread_count = db.get_feed_unread_count(feed.id.as_str()).unwrap();
            tree.insert_item(
                TreeEntry {
                    id: feed.id,
                    title: feed.title,
                    unread_count: Some(unread_count),
                },
                Placement::LastChild,
                1,
            );
        }
    }

    // FIXME: hack to properly align elements in tree view
    if tree.len() > 1 {
        tree.remove_item(0);
    }
}

fn content_select_down(s: &mut Cursive) {
    s.call_on_name("content", move |view: &mut SelectView<Article>| {
        view.select_down(1)
    });
}

fn content_select_up(s: &mut Cursive) {
    s.call_on_name("content", move |view: &mut SelectView<Article>| {
        view.select_up(1)
    });
}

fn tree_on_collapse(siv: &mut Cursive, row: usize, collapsed: bool, _children: usize) {
    if !collapsed {
        let db = DB::new();
        let value = siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
            tree.borrow_item(row).unwrap().clone()
        });
        let v = value.unwrap_or(TreeEntry::default());
        let articles = db.get_articles_for_category(&v.id).unwrap();

        draw_articles(articles, siv, &v.title);
    }
}

fn draw_articles(articles: Vec<Article>, siv: &mut Cursive, title: &str) {
    siv.call_on_name("panel", move |view: &mut Dialog| {
        view.set_title(title);
    });
    siv.call_on_name("content", move |view: &mut SelectView<Article>| {
        view.clear();
        for article in articles {
            view.add_item(article.draw(), article);
        }
    });

    siv.focus_name("content").unwrap();
}

fn decrease_unread_count(tree: &mut TreeView<TreeEntry>, row: usize) {
    let item = tree.borrow_item_mut(row).unwrap();
    if let Some(count) = item.unread_count {
        if count > 0 {
            item.unread_count = Some(item.unread_count.unwrap() - 1);
        }
    }
}
