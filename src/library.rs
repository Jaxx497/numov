#![allow(dead_code)]
use rusqlite::Result;
use std::{collections::HashMap, path::PathBuf, u32};
use walkdir::WalkDir;

use crate::{database::Database, movie::Movie};

#[derive(Debug)]
pub struct Library {
    db: Database,
    root: String,
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
        let existing = db.fetch_movies().unwrap();

        Library {
            db,
            root: root.to_string(),
            new: vec![],
            collection: vec![],
            existing,
        }
    }

    pub fn build(&mut self) -> Result<()> {
        // ROADMAP FOR THIS FUNCTION
        // 1. Add any new additions to the database
        // 2. Remove all updated or deleted files
        // 3. Create `collection`

        // Get list of paths
        // For each path, get a hash of the file
        // If the file is in the existing hashes
        //      add movie to [collection]
        //  else
        //      Create a new movie, add to [new]

        let path_list = Self::_get_dirs(&self.root);
        // let path_list = Self::_get_dirs(&self.root)[127..135].to_vec();

        for path in path_list {
            let (_, hash) = numov::read_metadata(&path);

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
                    println!("ADDED MOVIES");
                    self.new.iter().for_each(|m| println!("\t{}", m.title));
                    self.collection.append(&mut self.new)
                }
                Err(e) => eprintln!("Error inserting movies: {e}"),
            }
        }

        if !self.existing.is_empty() {
            println!("REMOVED MOVIES");
            self.existing
                .values()
                .for_each(|m| println!("\t{}", m.title));
            self.db.bulk_removal(&self.existing)?;
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
