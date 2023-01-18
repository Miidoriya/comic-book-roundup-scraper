use core::result::Result;
use fuzzywuzzy::fuzz;
use inquire::{Select, Text};
use reqwest::Error;
use scraper::{error::SelectorErrorKind, Html};
use serde::{Deserialize, Serialize};
use cli_table::{format::Justify, print_stdout, Table, WithTitle};

#[derive(Debug)]
enum PublisherError<'a> {
    RequestError(reqwest::Error),
    SelectorError(SelectorErrorKind<'a>),
}

impl<'a> From<reqwest::Error> for PublisherError<'a> {
    fn from(error: reqwest::Error) -> Self {
        PublisherError::RequestError(error)
    }
}

impl<'a> From<SelectorErrorKind<'a>> for PublisherError<'a> {
    fn from(error: SelectorErrorKind<'a>) -> Self {
        PublisherError::SelectorError(error)
    }
}

#[derive(Debug)]
struct Publisher {
    name: String,
    url: String,
}

impl Publisher {
    fn new(name: String, url: String) -> Self {
        Publisher { name, url }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Title {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Table)]
struct Issue {
    #[table(title = "Title", justify = "Justify::Right")]
    title: String,
    #[table(title = "Issue Number")]
    issue_num: String,
    #[table(title = "Writer/s")]
    writer: String,
    #[table(title = "Artist/s")]
    artist: String,
    #[table(title = "User Review Score")]
    user_review: String,
    #[table(title = "Critic Review Score")]
    critic_review: String,
    #[table(title = "User Review Count")]
    user_review_count: String,
    #[table(title = "Critic Review Count")]
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

fn create_selector(selector_string: &str) -> scraper::Selector {
    scraper::Selector::parse(selector_string).unwrap()
}

fn do_request(url: &str) -> Result<String, Error> {
    let response = reqwest::blocking::get(url)?;
    response.text()
}

fn get_publishers() -> Result<Vec<Publisher>, PublisherError<'static>> {
    let url = "https://comicbookroundup.com/comic-books/reviews";
    let response = do_request(&url)?;
    let parsed_response = scraper::Html::parse_document(&response);
    let publisher_selector =
        scraper::Selector::parse("div.section > table > tbody > tr .top-publisher a")?;
    let publishers = parsed_response
        .select(&publisher_selector)
        .collect::<Vec<scraper::ElementRef>>();
    let publishers_vec = publishers
        .iter()
        .map(|publisher| {
            let publisher_elem = match publisher.value().attr("href") {
                Some(href) => href,
                None => {
                    println!("Error: {:?}", publisher);
                    return Publisher::new("".to_string(), "".to_string()); // How can we handle this better?
                }
            };
            let publisher_url = format!("https://comicbookroundup.com{}/{}", publisher_elem, "all-series");
            let publisher = Publisher::new(
                publisher_elem.split("/").last().unwrap().to_string(),
                publisher_url,
            );
            publisher
        })
        .collect::<Vec<Publisher>>();
    Ok(publishers_vec)
}

fn all_issues_document(series_url_string: &str) -> Result<scraper::Html, Error> {
    let url = format!("https://comicbookroundup.com{}", series_url_string);
    let response = do_request(&url)?;
    Ok(scraper::Html::parse_document(&response))
}

fn menu(items: &[String], msg: String) -> String {
    Select::new(&msg, items.to_vec()).prompt().unwrap()
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

fn main() {
    loop {
        match menu(
            &["Scrape publisher".into(), "Exit!".into()],
            "MENU".to_owned(),
        )
        .as_str()
        {
            "Scrape publisher" => {
                let publishers = get_publishers().unwrap();
                let mut publisher_names = publishers
                    .iter()
                    .map(|publisher| publisher.name.clone())
                    .collect::<Vec<String>>();
                publisher_names.push("Exit!".to_string());
                loop {
                    match menu(
                        publisher_names.as_slice(),
                        "Which publisher would you like to scrape?".to_owned(),
                    )
                    .as_str()
                    {
                        "Exit!" => break,
                        publisher => {
                            let current_publisher = publishers.iter().find(|publisher_obj| publisher_obj.name == publisher).unwrap();
                            println!("Scraping publisher: {}", current_publisher.url);
                            let publisher_doc = do_request(&current_publisher.url).unwrap();
                            let publisher_parsed: Result<Html, Error> = Ok(scraper::Html::parse_document(&publisher_doc));
                            let document = match publisher_parsed {
                                Ok(document) => document,
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }};
                            let title_name = Text::new("Which title are you looking for?:")
                                .prompt()
                                .unwrap();
                            let title_selector = match scraper::Selector::parse("td.series>a") {
                                Ok(selector) => selector,
                                Err(e) => {
                                    println!("Error: {}", e);
                                    continue;
                                }
                            };
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
                                format!("Which {} comic would you like to scrape?", &title_name),
                            )
                            .as_str()
                            {
                                named_title => {
                                    let title = titles
                                        .iter()
                                        .find(|title| title.name == named_title)
                                        .unwrap();
                                    let document = match all_issues_document(&title.url) {
                                        Ok(doc) => doc,
                                        Err(e) => {
                                            println!("Error: {}", e);
                                            continue;
                                        }
                                    };
                                    let issue_selector =
                                        create_selector("div.section > table > tbody > tr");
                                    let mut issues = document
                                        .select(&issue_selector)
                                        .collect::<Vec<scraper::ElementRef>>();
                                    issues.drain(0..1);
                                    let critic_rating_selector =
                                        create_selector(".rating .CriticRatingList div");
                                    let user_rating_selector =
                                        create_selector(".rating .UserRatingList div");
                                    let critic_reviews_count_selector =
                                        create_selector(".reviews .CriticReviewNumList a");
                                    let user_reviews_count_selector =
                                        create_selector(".reviews .UserReviewNumList a");
                                    let issue_number_selector = create_selector(".issue a");
                                    let url_selector = create_selector(".issue a");
                                    let writer = create_selector(".writer a");
                                    let artist = create_selector(".artist a");
                                    let issues = issues.iter().map(|issue| {
                                        let title = named_title.to_string();
                                        let issue_num =
                                            match issue.select(&issue_number_selector).next() {
                                                Some(issue_num) => issue_num.inner_html(),
                                                None => "N/A".to_string(),
                                            };
                                        let writer = match issue.select(&writer).next() {
                                            Some(writer) => writer.inner_html(),
                                            None => "N/A".to_string(),
                                        };
                                        let artist = match issue.select(&artist).next() {
                                            Some(artist) => artist.inner_html(),
                                            None => "N/A".to_string(),
                                        };
                                        let critic_review =
                                            match issue.select(&critic_rating_selector).next() {
                                                Some(critic_rating) => critic_rating.inner_html(),
                                                None => "N/A".to_string(),
                                            };
                                        let user_review =
                                            match issue.select(&user_rating_selector).next() {
                                                Some(user_rating) => user_rating.inner_html(),
                                                None => "N/A".to_string(),
                                            };
                                        let critic_review_count = match issue
                                            .select(&critic_reviews_count_selector)
                                            .next()
                                        {
                                            Some(critic_reviews_count) => {
                                                critic_reviews_count.inner_html()
                                            }
                                            None => "N/A".to_string(),
                                        };
                                        let user_review_count =
                                            match issue.select(&user_reviews_count_selector).next()
                                            {
                                                Some(user_reviews_count) => {
                                                    user_reviews_count.inner_html()
                                                }
                                                None => "N/A".to_string(),
                                            };
                                        let issue = Issue::new(
                                            title,
                                            issue_num,
                                            writer,
                                            artist,
                                            user_review,
                                            critic_review,
                                            user_review_count,
                                            critic_review_count,
                                        );
                                        issue
                                    });
                                    let table = issues.collect::<Vec<Issue>>();
                                    print_stdout(table.with_title());
                                }
                            }
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
