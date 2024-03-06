#![allow(dead_code)]
use rusqlite::Result;
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;
use xxhash_rust::const_xxh32::xxh32;

use crate::{database::Database, movie::Movie};

#[derive(Debug)]
pub struct Library<'a> {
    db: &'a Database,
    root: String,
    pub new: Vec<Movie>,
    pub collection: Vec<Movie>,
    pub existing: HashSet<u32>,
}

impl<'a> Library<'a> {
    pub fn new(root: &str, db: &'a Database) -> Self {
        let existing = Self::get_existing(db);
        Library {
            db,
            root: root.to_string(),
            new: vec![],
            collection: vec![],
            existing,
        }
    }

    // pub fn get_existing_ye(&mut self) {
    //     self.existing = self
    //         .db
    //         .conn
    //         .prepare("SELECT hash FROM movies")
    //         .expect("Failed on SELECT statement")
    //         .query_map([], |row| row.get(0))
    //         .expect("Failed to read rows")
    //         .collect::<Result<_>>()
    //         .expect("Failed to create hashmap");
    // }

    pub fn get_existing(db: &'a Database) -> HashSet<u32> {
        db.conn
            .prepare("SELECT hash FROM movies")
            .expect("Failed on SELECT statement")
            .query_map([], |row| row.get(0))
            .expect("Failed to read rows")
            .collect::<Result<_>>()
            .expect("Failed to create hashmap")
    }

    pub fn build(&mut self) -> Result<()> {
        // let path_list = Self::_get_dirs(&self.root)[20..25].to_vec();
        let path_list = Self::_get_dirs(&self.root);

        self.db.conn.execute("BEGIN TRANSACTION", [])?;

        for path in path_list {
            let bytes = path.metadata().unwrap().len();
            let last_mod = path
                .metadata()
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Could not convert to timestamp.")
                .as_nanos();

            let mash = bytes as u128 + last_mod;
            let hash = xxh32(&mash.to_be_bytes(), 0);

            // If the hash of the file we're reading
            // is NOT in the existing known hashes
            if !self.existing.contains(&hash) {
                // Create a movie obj
                let mov = Movie::new(path);

                // Add that movie to the database
                match self.db.conn.execute(
                    "INSERT INTO movies (title, hash, size) VALUES (?, ?, ?)",
                    (&mov.title.to_string_lossy(), &mov.hash, &mov.size),
                ) {
                    Ok(_) => {
                        self.new.push(mov);
                    }
                    Err(err) if err.to_string().contains("UNIQUE constraint failed") => {
                        self.existing.remove(&hash);
                    }
                    Err(err) => eprintln!("{:?}", err),
                }
            } else {
                self.existing.remove(&hash);
            }
        }
        self.db.conn.execute("COMMIT", []).unwrap();
        Ok(())
    }

    fn _get_dirs(root: &str) -> Vec<PathBuf> {
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
