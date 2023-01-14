use inquire::{Select, Text};
use reqwest::Error;

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
        match menu(&["Scrape publisher".into(), "Do Nothing".into()]).as_str() {
            "Scrape publisher" => {
                let publisher = Text::new("Enter your publisher:").prompt().unwrap();
                let document = all_series_document(&publisher).unwrap();

                let title_selector = scraper::Selector::parse("td.series>a").unwrap();

                let titles_vec: Vec<String> = document
                    .select(&title_selector)
                    .map(|x| x.inner_html())
                    .collect();
                let titles_count = titles_vec.len();
                println!("The number of titles: {}", titles_count);
                titles_vec
                    .iter()
                    .zip(1..titles_count + 1)
                    .for_each(|(item, number)| println!("{}. {}", number, item));
                
                continue;
            }
            "Do Nothing" => {
                continue;
            }
            _ => println!("default"),
        }
    }
}
