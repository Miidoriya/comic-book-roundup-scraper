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

struct Issue {
    title: String,
    issue_num: String,
    writer: String,
    artist: String,
    user_review: String,
    critic_review: String,
    user_review_count: String,
    critic_review_count: String,
}

impl Title {
    fn new(name: String, url: String) -> Self {
        Title { name, url }
    }
}

impl Issue {
    fn new(
        title: String,
        issue_num: String,
        writer: String,
        artist: String,
        user_review: String,
        critic_review: String,
        user_review_count: String,
        critic_review_count: String,
    ) -> Self {
        Issue {
            title,
            issue_num,
            writer,
            artist,
            user_review,
            critic_review,
            user_review_count,
            critic_review_count,
        }
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

fn all_issues_document(series_url_string: &str) -> Result<scraper::Html, Error> {
    let url = format!("https://comicbookroundup.com{}", series_url_string);
    let response = do_request(&url)?;
    Ok(scraper::Html::parse_document(&response))
}

fn menu(items: &[String]) -> String {
    Select::new("MENU", items.to_vec()).prompt().unwrap()
}

fn calculate_title_similarity(title_name: &str, title: &scraper::ElementRef) -> u32 {
    fuzz::ratio(&title.inner_html(), &title_name).into()
}

fn find_titles(title_name: &str, titles: &[scraper::ElementRef]) -> Vec<Title> {
    titles
        .into_iter()
        .filter(|title| calculate_title_similarity(title_name, title) > 50)
        .map(|title| {
            Title::new(
                title.inner_html(),
                title.value().attr("href").unwrap().to_string(),
            )
        })
        .collect()
}

fn print_title_info(title: &str, url: &str) {
    println!("Title: {}", title);
    println!("URL: {}", url);
}

fn main() {
    loop {
        match menu(&["Scrape publisher".into(), "Exit!".into()]).as_str() {
            "Scrape publisher" => {
                let publisher = Text::new("Enter your publisher:").prompt().unwrap();
                let document = match all_series_document(&publisher) {
                    Ok(doc) => doc,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };

                let title_selector = match scraper::Selector::parse("td.series>a") {
                    Ok(selector) => selector,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };

                loop {
                    let title_name = Text::new("Which title are you looking for?:")
                        .prompt()
                        .unwrap();

                    let titles = find_titles(
                        &title_name,
                        &document
                            .select(&title_selector)
                            .collect::<Vec<scraper::ElementRef>>(),
                    );

                    match menu(
                        &titles
                            .iter()
                            .map(|title| title.name.clone())
                            .collect::<Vec<String>>(),
                    )
                    .as_str()
                    {
                        named_title => {
                            let title = titles
                                .iter()
                                .find(|title| title.name == named_title)
                                .unwrap();
                            let issue_document = match all_issues_document(&title.url) {
                                Ok(doc) => doc,
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }
                            };
                            let issue_selector = match scraper::Selector::parse(
                                "div.section > table > tbody > tr",
                            ) {
                                Ok(selector) => selector,
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }
                            };

                            let title_selector = match scraper::Selector::parse(
                                ".rating .CriticRatingList div",
                            ) {
                                Ok(selector) => selector,
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }
                            };

                            let mut issues = issue_document
                                .select(&issue_selector)
                                .collect::<Vec<scraper::ElementRef>>();
                            issues.drain(0..1);

                            issues.iter().for_each(|issue| {;
                                let title = issue.select(&title_selector).next().unwrap().inner_html();
                                println!("{:?}", title);
                            });
                            // let formatted_issues = for ele in issues {
                            //     Issue::new(title, issue_num, writer, artist, user_review, critic_review, user_review_count, critic_review_count)
                            // };

                            // for ele in issues {
                            //     println!("{:?}", ele.inner_html());
                            // }
                            // let title_info = issues
                            //     .iter()
                            //     .map(|title| title.text());

                            // title_info.for_each(|title| println!("{:?}", title));
                            //println!("{:?}", title_info);
                        }
                    }
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
