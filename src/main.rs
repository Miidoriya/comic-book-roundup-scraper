use core::result::Result;
use fuzzywuzzy::fuzz;
use inquire::{Select, Text};
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
        match menu(&["Scrape publisher".into(), "Exit!".into()]).as_str() {
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
                        .filter(|title| fuzz::ratio(&title.name, &title_name) > 50) // 2 is the threshold for similarity
                        .collect();

                    match menu(titles_vec.iter().map(|x| x.name.clone()).collect::<Vec<String>>().as_slice()).as_str() {
                        named_title => {
                            let title = titles_vec
                                .iter()
                                .find(|title| title.name == named_title)
                                .unwrap();
                            println!("Title: {}", title.name);
                            println!("URL: {}", title.url);
                        }
                        "Exit!" => break,
                        _ => println!("default"),
                    }
                    continue;
                }
            }
            "Exit!" => {
                println!("Exiting CLI interface ...");
                break;
            }
            _ => println!("default"),
        }
    }
}
