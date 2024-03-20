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
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Library {
    pub db: Database,
    pub root: PathBuf,
    pub legacy_collection: HashSet<u32>,
    pub collection: HashMap<u32, Movie>,
    pub ratings: HashMap<String, String>,
}

impl Library {
    pub fn new(root: PathBuf) -> Self {
        let db = Database::open().unwrap_or_else(|e| {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        });

        let (collection, ratings) = db.fetch();
        if !collection.is_empty() || !ratings.is_empty() {
            println!(
                "Read in {} movies and {} ratings from database.",
                collection.len(),
                ratings.len()
            );
        }
        let legacy_collection = collection.keys().cloned().collect::<HashSet<u32>>();

        Library {
            db,
            root,
            ratings,
            collection,
            legacy_collection,
        }
    }

    // For each .mkv in the path_list
    //  Generate hash, and try to remove it from hashet
    //  If it cannot be removed:
    //      Create a new movie instance, and add it to collection
    pub fn update_movies(&mut self) -> Result<()> {
        if !PathBuf::from(&self.root).is_dir() {
            println!("Improper path provided.");
            return Ok(());
        }

        // self.legacy_collection = self.collection.keys().cloned().collect();
        let mut logger: HashMap<String, MovieStatus> = HashMap::new();
        let path_list = Self::_get_dirs(&self.root);

        let mut main_prog = Prog::new(path_list.len(), "updated library");
        for path in &path_list {
            let hash = Movie::read_metadata(path).1;
            if !self.legacy_collection.remove(&hash) {
                let movie = Movie::new(path);
                logger.insert(
                    format!("{} ({})", &movie.title, movie.year),
                    MovieStatus::New,
                );
                self.collection.insert(hash, movie);
            }
            main_prog.inc();
        }
        main_prog.end();

        self.map_ratings();

        self.legacy_collection.iter().for_each(|bad_hash| {
            if let Some(m) = self.collection.remove(bad_hash) {
                logger
                    .entry(format!("{} ({})", &m.title, m.year))
                    .and_modify(|t| *t = MovieStatus::Updated)
                    .or_insert(MovieStatus::Removed);
            };
        });

        if !logger.is_empty() {
            self.db
                .update_movie_table(&self.collection, &self.legacy_collection)
                .unwrap_or_else(|e| println!("Failed to update the database.\nError: {e}"));
        }
        Self::log_changes(logger);

        Ok(())
    }

    /// Given a `user_name` (String) from letterboxd, scrape ratings and store in database
    pub fn update_ratings(&mut self, user_name: &impl AsRef<str>) -> Result<()> {
        let ratings = Self::retrieve_ratings(user_name.as_ref());

        match self.db.update_ratings_table(&ratings) {
            Ok(_) => {
                println!("ADDED {} RATINGS!", ratings.len());
                self.ratings = ratings;
            }
            Err(e) => println!("Could not scrape ratings!\nError: {e}"),
        };
        Ok(())
    }
}

// ==================
// RATINGS RELATED
// ==================
impl Library {
    fn retrieve_ratings(user_name: impl AsRef<str>) -> HashMap<String, String> {
        let url = format! {"https://letterboxd.com/{}/films/", user_name.as_ref()};
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

// =========================
// External Functionality
// =========================
impl Library {
    pub fn output_to_csv(&self) {
        let mut wtr = csv::Writer::from_path("m_log.csv").unwrap();
        let mut csv_prog = Prog::new(self.collection.len(), "writing movies to csv");

        wtr.serialize([
            "Title",
            "Year",
            "Rating",
            "Duration",
            "Size",
            "Resolution",
            "V_Coec",
            "Bit_depth",
            "A_Codec",
            "Channels",
            "Sub_Format",
            "Hash",
            "Audio #",
            "Sub #",
        ])
        .ok();

        self.collection.values().for_each(|m| {
            wtr.serialize((
                &m.title,
                &m.year,
                &m.rating,
                &m.duration,
                format!("{:.2}", m.size),
                &m.video.resolution.to_string(),
                &m.video.codec,
                &m.video.bit_depth.to_string(),
                &m.audio.codec.to_string(),
                &m.audio.channels,
                &m.subs.format,
                format!("{:x}", &m.hash),
                &m.audio.count,
                &m.subs.count,
            ))
            .unwrap_or_else(|e| println!("Error writing {} to csv with error: {e}", m.title));
            csv_prog.inc();
        });
        csv_prog.end();
    }

    pub fn rename_folders(&mut self) {
        let mut old_hashes = HashSet::new();

        for path in &Self::_get_dirs(&self.root) {
            let hash = Movie::read_metadata(path).1;

            if let Some(m) = self.collection.get(&hash) {
                let old_name = path.parent().unwrap();
                let new_name = self.get_new_name(m);
                let file_name = path.file_name().unwrap();

                if new_name != old_name {
                    old_hashes.insert(hash);
                    let mut m = self.collection.remove(&hash).unwrap();

                    std::fs::rename(old_name, &new_name).unwrap_or_else(|e| {
                        println!("Error writing to {:?}\nError: {e}", &new_name)
                    });

                    let new_path = &new_name.join(file_name);
                    let new_hash = Movie::read_metadata(new_path).1;
                    m.hash = new_hash;
                    self.collection.insert(new_hash, m);

                    println!(
                        "\n\t\t{}\n\t\t==>\t{}",
                        old_name.display(),
                        new_name.display(),
                    )
                }
            }
        }

        if !old_hashes.is_empty() {
            println!("\nRenamed {} paths!", old_hashes.len());

            self.db
                .update_movie_table(&self.collection, &old_hashes)
                .unwrap_or_else(|e| println!("Failed to update the database.\nError: {e}"));
        }
    }

    fn get_new_name(&self, m: &Movie) -> PathBuf {
        let new_path = format!(
            "{}{} ({}) [{} {} {} {:?}-{}] ({:.2} GB)",
            self.root.to_string_lossy(),
            m.title,
            m.year,
            m.video.resolution,
            m.video.codec,
            m.video.bit_depth,
            m.audio.codec,
            match m.audio.channels {
                ch if ch < 1.5 => "mono".to_string(),
                ch if ch < 2.5 => "stereo".to_string(),
                x => format!("{}", &x),
            },
            m.size
        );
        PathBuf::from(new_path)
    }
}

// =================
// Private Stuff
// =================
impl Library {
    fn _get_dirs(root: &PathBuf) -> Vec<PathBuf> {
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

    fn map_ratings(&mut self) {
        let mut count = 0;
        for movie in self.collection.values_mut() {
            let mut best_match = (0.8, None);

            for (rating_title, rating_value) in &self.ratings {
                let similarity = strsim::jaro_winkler(&movie.title, rating_title);

                if similarity > best_match.0 {
                    best_match = (similarity, Some(rating_value.clone()))
                }
            }
            if best_match.0 > 0.84 {
                movie.rating = best_match.1;
                count += 1;
            }
        }
        if count > 0 {
            println!("Successfully mapped ratings to {count} movies.")
        }
    }

    fn log_changes(logger: HashMap<String, MovieStatus>) {
        let mut log = vec![vec![], vec![], vec![]];
        for (k, v) in logger {
            match v {
                MovieStatus::New => log[0].push(k),
                MovieStatus::Updated => log[1].push(k),
                MovieStatus::Removed => log[2].push(k),
            }
        }

        if !log[0].is_empty() || !log[1].is_empty() || !log[2].is_empty() {
            println!(
                "\nADDED {} | UPDATED {} | REMOVED {}",
                log[0].len(),
                log[1].len(),
                log[2].len()
            );
        }

        for (i, m) in ["++", "**", "--"].into_iter().enumerate() {
            if !log[i].is_empty() {
                match log[i].len() {
                    t if t < 15 => log[i].iter().for_each(|title| println!("\t{m} {title}")),
                    _ => println!("\t{}", log[i].join("  |  ")),
                }
            }
        }
    }
}

#[derive(Debug)]
enum MovieStatus {
    New,
    Removed,
    Updated,
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
            "\tFinished {} in {:.4?}",
            self.job,
            Instant::now() - self.t1
        );
        self.pb.finish_println(&output);
        println!();
    }
}
