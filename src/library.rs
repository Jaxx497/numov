#![allow(dead_code)]
use rusqlite::{Result, ToSql};
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

use crate::{database::Database, movie::Movie};

#[derive(Debug)]
pub struct Library<'a> {
    db: &'a Database,
    root: String,
    pub new: Vec<String>,
    pub existing: HashSet<u32>,
    pub collection: Vec<Movie>,
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

    pub fn get_existing(db: &'a Database) -> HashSet<u32> {
        db.conn
            .prepare("SELECT hash FROM movies")
            .expect("Failed on SELECT statement")
            .query_map([], |row| row.get(0))
            .expect("Failed to read rows")
            .collect::<Result<HashSet<u32>>>()
            .expect("Failed to create hashmap of existing values!")
    }

    pub fn build(&mut self) -> Result<()> {
        // ROADMAP FOR THIS FUNCTION
        // 1. Add any new additions to the database
        // 2. Remove all updated or deleted files
        // 3. Create `collection`

        let path_list = Self::_get_dirs(&self.root)[20..25].to_vec();
        // let path_list = Self::_get_dirs(&self.root);

        self.db.conn.execute("BEGIN TRANSACTION", [])?;

        for path in path_list {
            let (_, hash) = numov::read_metadata(&path);

            if !self.existing.contains(&hash) {
                let mov = Movie::new(&path);

                match self.db.conn.execute(
                    "INSERT INTO movies (title, year, size, duration, resolution, v_codec, bit_depth, a_codec, channels, a_count, sub_format, sub_count, hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )",
                    (&mov.title,
                        &mov.year,
                        format!("{:.2}", &mov.size),
                        &mov.duration,
                        &mov.video.resolution,
                        &mov.video.codec,
                        &mov.video.bit_depth,
                        &mov.audio.codec,
                        &mov.audio.channels.to_string(),
                        &mov.audio.count,
                        &mov.subs.format,
                        &mov.subs.count,
                        &mov.hash,
                    ),
                ) {
                    Ok(_) => {
                        self.new.push(format!("{} ({})", mov.title, mov.year));
                        self.collection.push(mov);
                    }
                    Err(err) => eprintln!("{:?}", err),
                    // Err(err) if err.to_string().contains("UNIQUE constraint failed") => {
                    //     self.existing.remove(&hash);
                    // }
                }
            }
        }
        self.db.conn.execute("COMMIT", []).unwrap();
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
