#![allow(dead_code)]
use crate::{database::Database, movie::Movie};
use rusqlite::Result;
use select::{
    document::Document,
    predicate::{Attr, Class},
};
use std::{collections::HashMap, path::PathBuf, u32};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Library {
    db: Database,
    root: String,
    pub ratings: HashMap<String, String>,
    pub existing: HashMap<u32, Movie>,
    pub new: Vec<Movie>,
    pub collection: Vec<Movie>,
}

impl Library {
    pub fn new(root: &str) -> Self {
        let db = match Database::new() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Could not make database!\nError: {e}");
                std::process::exit(1);
            }
        };

        let (existing, ratings) = db.fetch_all();
        if !existing.is_empty() || !ratings.is_empty() {
            println!(
                "Read in {} movies and {} ratings from database.",
                existing.len(),
                ratings.len()
            );
        }

        Library {
            db,
            root: root.to_string(),
            new: vec![],
            ratings,
            collection: vec![],
            existing,
        }
    }

    /// Run existing files against database entries
    /// and build `self.collection` while updating
    /// database to reflect the provided root
    pub fn update_movies(&mut self) -> Result<()> {
        // let path_list = Self::_get_dirs(&self.root);
        let path_list = Self::_get_dirs(&self.root)[130..220].to_vec();

        for path in path_list {
            let (_, hash) = Movie::read_metadata(&path);

            match self.existing.remove(&hash) {
                Some(m) => self.collection.push(m),
                None => {
                    let movie = Movie::new(&path);
                    self.new.push(movie);
                }
            }
        }

        if !self.new.is_empty() {
            match self.db.bulk_insert(&self.new) {
                Ok(()) => {
                    println!("\nADDED {} MOVIES", self.new.len());

                    if self.new.len() < 20 {
                        self.new.iter().for_each(|m| println!("\t{}", m.title));
                    }

                    self.collection.append(&mut self.new)
                }
                Err(e) => eprintln!("Error inserting movies: {e}"),
            }
        }

        if !self.existing.is_empty() {
            println!("\nREMOVED {} MOVIES", self.existing.len());

            if self.existing.len() < 21 {
                self.existing
                    .values()
                    .for_each(|m| println!("\t{}", m.title));
            }

            self.db.bulk_removal(&self.existing)?;
        }

        self.collection
            .sort_unstable_by(|a, b| a.title.cmp(&b.title));

        Ok(())
    }

    pub fn update_ratings(&mut self) -> Result<()> {
        let ratings = Self::retrieve_ratings();

        match self.db.update_ratings(&ratings) {
            Ok(_) => {
                println!("\tAdded {} ratings!", ratings.len());
                self.ratings = ratings;
            }
            Err(e) => println!("Could not scrape ratings!\nError: {e}"),
        };
        Ok(())
    }

    pub fn _get_dirs(root: &str) -> Vec<PathBuf> {
        WalkDir::new(root)
            .max_depth(2)
            .into_iter()
            .filter_map(|file| {
                file.ok().and_then(
                    |entry| match entry.path().to_string_lossy().ends_with("mkv") {
                        true => Some(entry.path().to_owned()),
                        false => None,
                    },
                )
            })
            .collect()
    }
}

// =====================
// RATINGS RELATED
// =====================
impl Library {
    fn retrieve_ratings() -> HashMap<String, String> {
        let url = "https://letterboxd.com/equus497/films/";
        let mut catalogue = HashMap::new();

        let doc = Self::get_document(url);

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

        Self::extract_info(&doc, &mut catalogue);

        while last_page > 1 {
            let doc = Self::get_document(&format!("{}/page/{}", url, last_page));
            Self::extract_info(&doc, &mut catalogue);
            last_page -= 1;
        }
        catalogue
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
}
