use news_rss::config::Config;
use news_rss::db::DB;
use news_rss::greader::Greader;
use news_rss::ui::UI;
use news_rss::utils;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(format!("{}/config.toml", utils::get_config_dir()));
    let config = Config::from(&path.display().to_string());
    match config {
        Ok(config) => {
            DB::new()
                .create_db()
                .expect("Something went wrong while creating DB");
            let greader = Greader::login(config.clone()).unwrap();
            let mut ui = UI::new();
            ui.create(greader, config);
        }
        Err(error) => println!(
            "Something went wrong while reading config.toml file:\n{:#}",
            error
        ),
    }
}
