use news_rss::category::Category;
use news_rss::db::DB;
use news_rss::ui::UI;

fn main() {
    DB::new()
        .create_db()
        .expect("Something went wrong while creating DB");
    let category_list = vec![
        Category::new("UaNews", vec![]),
        Category::new("Telegram", vec![]),
    ];
    let mut ui = UI::new();
    ui.create(category_list);
}
