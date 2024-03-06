use crate::Database;
use select::{
    document::Document,
    predicate::{Attr, Class},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Ratings<'a> {
    db: &'a Database,
    pub ratings_raw: HashMap<String, String>,
}

impl<'a> Ratings<'a> {
    pub fn new(db: &'a Database) -> Self {
        Ratings {
            db,
            ratings_raw: HashMap::new(),
        }
    }

    pub fn insert_to_table(&self) {
        self.db
            .conn
            .execute("BEGIN TRANSACTION", [])
            .expect("Could not beging transaction on RATINGS");

        let ratings_table = retrieve_ratings();

        for (key, value) in ratings_table {
            self.db
                .conn
                .execute(
                    "INSERT OR REPLACE INTO ratings (title, rating) VALUES (?, ?)",
                    (&key, &value),
                )
                .expect("Could not insert ratings");
        }
        self.db
            .conn
            .execute("COMMIT", [])
            .expect("Could not COMMIT changes to ratings table");
    }
}

fn get_document(url: &str) -> Document {
    let req = ureq::get(url)
        .call()
        .expect("Failed to make request.")
        .into_string()
        .expect("Failed to convert response to String.");

    Document::from(req.as_str())
}

fn extract_info(doc: &Document, catalogue: &mut HashMap<String, String>) {
    for poster in doc.find(Class("poster-container")) {
        let title = poster
            .find(Attr("alt", ()))
            .into_selection()
            .first()
            .unwrap()
            .attr("alt")
            .expect("Could not find alt tag.")
            .replace(':', "-");

        let rating = poster.text().trim().to_string();

        if let Some(existing_rating) = catalogue.get(&title) {
            if existing_rating.len() > rating.len() {
                continue;
            }
        }
        catalogue.insert(title, rating);
    }
}

fn retrieve_ratings() -> HashMap<String, String> {
    let url = "https://letterboxd.com/equus497/films/";
    let mut catalogue = HashMap::new();

    let doc = get_document(url);

    let mut last_page = match doc.find(Class("paginate-pages")).into_selection().first() {
        Some(n) => n
            .text()
            .split_whitespace()
            .last()
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        None => 1,
    };

    extract_info(&doc, &mut catalogue);

    while last_page > 1 {
        let doc = get_document(&format!("{}/page/{}", url, last_page));
        extract_info(&doc, &mut catalogue);
        last_page -= 1;
    }
    catalogue
}
