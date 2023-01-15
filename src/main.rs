use core::result::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use inquire::{Select, Text};
use levenshtein::levenshtein;
use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Title {
    name: String,
    url: String,
}

impl Title {
    fn new(name: String, url: String) -> Self {
        Title { name, url }
    }
}

fn do_request(url: &str) -> Result<String, Error> {
    let response = reqwest::blocking::get(url)?;
    response.text()
}

fn all_series_document(publisher: &str) -> Result<scraper::Html, Error> {
    let url = format!(
        "https://comicbookroundup.com/comic-books/reviews/{}/all-series",
        publisher
    );
    let response = do_request(&url)?;
    Ok(scraper::Html::parse_document(&response))
}

fn menu(items: &[String]) -> String {
    Select::new("MENU", items.to_vec()).prompt().unwrap()
}

fn main() {
    loop {
        match menu(&[
            "Scrape publisher".into(),
            "Do Nothing".into(),
            "Exit!".into(),
        ])
        .as_str()
        {
            "Scrape publisher" => {
                let publisher = Text::new("Enter your publisher:").prompt().unwrap();
                let document = all_series_document(&publisher).unwrap();

                let title_selector = scraper::Selector::parse("td.series>a").unwrap();

                loop {
                    let title_name = Text::new("Which title are you looking for?:")
                        .prompt()
                        .unwrap();

                    let titles_vec: Vec<Title> = document
                        .select(&title_selector)
                        .map(|x| {
                            Title::new(x.inner_html(), x.value().attr("href").unwrap().to_string())
                        })
                        .into_iter()
                        .filter(|title| levenshtein(&title.name, &title_name) <= 7) // 2 is the threshold for similarity
                        .collect();

                    let titles_count = titles_vec.len();
                    titles_vec
                        .iter()
                        .zip(1..titles_count + 1)
                        .for_each(|(title, number)| {
                            println!("{}. {}", number, serde_json::to_string(&title).unwrap())
                        });
                    continue;
                }
            }
            "Do Nothing" => {
                continue;
            }
            "Exit!" => {
                println!("Exiting CLI interface ...");
                break;
            }
            _ => println!("default"),
        }
    }
}
