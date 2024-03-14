#![allow(dead_code)]
use crate::{database::Database, movie::Movie};
use rusqlite::Result;
use select::{
    document::Document,
    predicate::{Attr, Class},
};
use std::{
    collections::{HashMap, HashSet},
    io::Stdout,
    path::PathBuf,
    time::Instant,
    u32,
};
use strsim::jaro_winkler;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Library {
    pub db: Database,
    root: String,
    new: Vec<String>,
    pub collection: HashMap<u32, Movie>,
    pub old_collection: HashSet<u32>,
    pub ratings: HashMap<String, String>,
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

        let (collection, ratings) = db.fetch_all();
        if !collection.is_empty() || !ratings.is_empty() {
            println!(
                "Read in {} movies and {} ratings from database.",
                collection.len(),
                ratings.len()
            );
        }

        let old_collection = collection.keys().cloned().collect::<HashSet<u32>>();

        Library {
            db,
            root: root.to_string(),
            new: vec![],
            ratings,
            collection,
            old_collection,
        }
    }

    pub fn update_movies(&mut self) -> Result<()> {
        let path_list = Self::_get_dirs(&self.root)[..170].to_vec();
        // let path_list = Self::_get_dirs(&self.root);
        let mut m_prog = Prog::new(path_list.len(), "updating moving list");
        for path in path_list {
            let (_, hash) = Movie::read_metadata(&path);
            if !self.old_collection.remove(&hash) {
                let movie = Movie::new(&path);
                self.new.push(format!("{} [{}]", &movie.title, &movie.hash));
                self.collection.insert(hash, movie);
                m_prog.inc();
            }
        }
        m_prog.end();

        if !self.ratings.is_empty() {
            match self.map_ratings() {
                Ok(()) => (),
                // Ok(()) => println!("UPDATED RATINGS FOR MOVIES"),
                Err(e) => println!("Error mapping ratings to movies. {e}"),
            }
        }

        if !self.new.is_empty() {
            match self.db.bulk_insert(&self.collection) {
                Ok(()) => {
                    println!("\nADDED {} MOVIES", self.new.len());

                    if self.new.len() < 20 {
                        self.new.iter().for_each(|m| println!("\t{}", m));
                    }
                }
                Err(e) => eprintln!("Error inserting movies: {e}"),
            }
        }

        // If any leftover values in `old_collection`...
        if !self.old_collection.is_empty() {
            println!("\nREMOVED {} MOVIES", self.old_collection.len());

            match self.old_collection.len() <= 15 {
                true => self.old_collection.iter().for_each(|bad_hash| {
                    if let Some(m) = self.collection.remove(bad_hash) {
                        println!("\t{} [{:x}]", m.title, m.hash);
                    }
                }),
                false => self.old_collection.iter().for_each(|bad_hash| {
                    self.collection.remove(bad_hash);
                }),
            }
            self.db.bulk_removal(&self.old_collection)?;
        }
        Ok(())
    }

    pub fn update_ratings(&mut self, user_name: &str) -> Result<()> {
        let ratings = Self::retrieve_ratings(user_name);

        match self.db.update_ratings(&ratings) {
            Ok(_) => {
                println!("ADDED {} RATINGS!", ratings.len());
                self.ratings = ratings;
            }
            Err(e) => println!("Could not scrape ratings!\nError: {e}"),
        };
        Ok(())
    }

    pub fn map_ratings(&mut self) -> Result<()> {
        for movie in self.collection.values_mut() {
            let mut best_match = (0.0, None);

            for (rating_title, rating_value) in &self.ratings {
                let similarity = jaro_winkler(&movie.title, rating_title);

                if similarity > best_match.0 {
                    best_match = (similarity, Some(rating_value.clone()))
                }
            }
            if best_match.0 >= 0.85 {
                movie.rating = best_match.1;
            }
        }
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
    fn retrieve_ratings(user_name: &str) -> HashMap<String, String> {
        let url = format! {"https://letterboxd.com/{}/films/", user_name};
        let mut catalogue = HashMap::new();

        let doc = Self::get_document(&url);

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

        let mut lb_prog = Prog::new(last_page, "scraping user ratings");
        lb_prog.pb.show_counter = false;
        while last_page > 1 {
            let doc = Self::get_document(&format!("{}/page/{}", url, last_page));
            Self::extract_info(&doc, &mut catalogue);
            last_page -= 1;
            lb_prog.inc();
        }
        lb_prog.end();
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

            if let Some(old_collection_rating) = catalogue.get(&title) {
                if old_collection_rating.len() > rating.len() {
                    continue;
                }
            }
            catalogue.insert(title, rating);
        }
    }
}

struct Prog {
    pub pb: pbr::ProgressBar<Stdout>,
    t1: Instant,
    job: String,
}
impl Prog {
    fn new(total: usize, job: &str) -> Self {
        Prog {
            pb: pbr::ProgressBar::new(total as u64),
            t1: Instant::now(),
            job: job.to_string(),
        }
    }

    fn inc(&mut self) {
        self.pb.inc();
    }

    fn end(&mut self) {
        let output = format!(
            "\tFinished {} in {:.4?}.",
            self.job,
            Instant::now() - self.t1
        );
        self.pb.finish_println(&output);
        println!();
    }
}
