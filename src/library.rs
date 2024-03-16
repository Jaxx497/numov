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
    // new: Vec<String>,
    // pub old_collection: HashSet<u32>,
    pub collection: HashMap<u32, Movie>,
    pub ratings: HashMap<String, String>,
}

impl Library {
    pub fn new(root: impl Into<String>) -> Self {
        let db = Database::new().unwrap_or_else(|e| {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        });

        let (collection, ratings) = db.fetch_all();
        if !collection.is_empty() || !ratings.is_empty() {
            println!(
                "Read in {} movies and {} ratings from database.",
                collection.len(),
                ratings.len()
            );
        }

        Library {
            db,
            root: root.into(),
            ratings,
            collection,
        }
    }

    // For each .mkv in the path_list
    //  Generate hash, and try to remove it from hashet
    //  If it cannot be removed:
    //      Create a new movie instance, and add it to collection
    pub fn update_movies(&mut self) -> Result<()> {
        let mut old_collection = self.collection.keys().cloned().collect::<HashSet<u32>>();
        let mut new = vec![];

        let path_list = Self::_get_dirs(&self.root);

        for path in &path_list[..40].to_vec() {
            let hash = Movie::read_metadata(path).1;
            if !old_collection.remove(&hash) {
                let movie = Movie::new(path);
                new.push(format!("{} ({})", movie.title.to_owned(), movie.year));
                self.collection.insert(hash, movie);
            }
        }

        // If data is in the `ratings` table, map ratings to collection items
        if !self.ratings.is_empty() {
            match self.map_ratings() {
                Ok(n) => println!("Successfully mapped ratings to {n} movies."),
                Err(e) => println!("Error mapping ratings to movies. {e}"),
            }
        }

        // If there are new movies,
        //  Write movies to database
        if !new.is_empty() {
            if let Err(e) = self.db.bulk_insert(&self.collection) {
                eprintln!("Error inserting movies into database.\nError: {e}")
            }
        }

        let mut logger: HashMap<String, ItemStatus> = new
            .into_iter()
            .map(|title| (title, ItemStatus::New))
            .collect();
        // If any leftover values in `old_collection`
        //  Remove them from the database
        // if !old_collection.is_empty() {
        let mut removed = vec![];
        if !old_collection.is_empty() {
            old_collection.iter().for_each(|bad_hash| {
                if let Some(m) = self.collection.remove(bad_hash) {
                    removed.push(format!("{} ({})", m.title.to_owned(), m.year));
                }
            });
            self.db.bulk_removal(&old_collection)?;
        }

        removed.into_iter().for_each(|title| {
            logger
                .entry(title)
                .and_modify(|t| *t = ItemStatus::Updated)
                .or_insert(ItemStatus::Removed);
        });

        for (k, v) in logger {
            match v {
                ItemStatus::New => println!("++ {k}"),
                ItemStatus::Updated => println!("** {k}"),
                ItemStatus::Removed => println!("-- {k}"),
            }
        }

        Ok(())
    }

    /// Given a `user_name` (String) from letterboxd, scrape ratings and store in database
    pub fn update_ratings(&mut self, user_name: &str) -> Result<()> {
        let ratings = Self::retrieve_ratings(user_name);

        match self.db.update_ratings_db(&ratings) {
            Ok(_) => {
                println!("ADDED {} RATINGS!", ratings.len());
                self.ratings = ratings;
            }
            Err(e) => println!("Could not scrape ratings!\nError: {e}"),
        };
        Ok(())
    }

    pub fn map_ratings(&mut self) -> Result<i32> {
        let mut count = 0;
        for movie in self.collection.values_mut() {
            let mut best_match = (0.0, None);

            for (rating_title, rating_value) in &self.ratings {
                let similarity = jaro_winkler(&movie.title, rating_title);

                if similarity > best_match.0 {
                    best_match = (similarity, Some(rating_value.clone()))
                }
            }
            if best_match.0 >= 0.85 {
                count += 1;
                movie.rating = best_match.1;
            }
        }
        Ok(count)
    }

    pub fn _get_dirs(root: impl AsRef<str>) -> Vec<PathBuf> {
        WalkDir::new(root.as_ref())
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

/////////////////////////////
// External Functionality
/////////////////////////////
impl Library {
    pub fn output_to_csv(&self) {
        // let mut wtr = csv::Writer::from_writer(io::stout());
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
                let file_name = path.file_name().unwrap();

                let new_name = self.get_new_name(m);

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

                    println!("\t\t{}\n\t==>\t{}", old_name.display(), new_name.display(),)
                }
            }
        }

        if !old_hashes.is_empty() {
            println!("Renamed {} paths!", old_hashes.len());
            self.db
                .bulk_removal(&old_hashes)
                .unwrap_or_else(|e| println!("Could not remove old instances from database! {e}"));

            self.db
                .bulk_insert(&self.collection)
                .unwrap_or_else(|e| println!("Could not update database! {e}"));
        }
    }

    fn get_new_name(&self, m: &Movie) -> PathBuf {
        let new_path = format!(
            "{}{} ({}) [{} {} {} {:?}-{}] ({:.2} GB)",
            self.root,
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

#[derive(Debug)]
enum ItemStatus {
    New,
    Removed,
    Updated,
}
