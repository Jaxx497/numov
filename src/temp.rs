use crate::database::Database;
use crate::movie::{Movie, VideoStream};
use crate::movie_types::resolution::Resolution;

use matroska::Matroska;
use rusqlite::params;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct MiniMovie {
    pub title: String,
    pub year: i16,
    // pub duration: String,
    pub hash: u32,
    pub duration: String,
    pub video: Resolution,
    // pub audio: AudioStream,
    // pub subs: SubtitleStream,
    pub size: f32,
}

pub fn build(db: &Database) {
    //     // ROADMAP FOR THIS FUNCTION
    //     // 1. Add any new additions to the database
    //     // 2. Remove all updated or deleted files
    //     // 3. Create `collection`
    //
    let path_list = _get_dirs("M:/")[103..110].to_vec();
    let mut existing = db.fetch_movies().unwrap();
    let mut collection: Vec<MiniMovie> = Vec::new();
    let mut new_movies: Vec<MiniMovie> = Vec::new();

    println!("{:?}", existing.len());
    println!("RUNNING CODE\n====================================================");
    for path in path_list {
        let (size, hash) = numov::read_metadata(&path);

        match existing.remove(&hash) {
            Some(m) => collection.push(m),
            _ => new_movies.push(MiniMovie {
                title: path.file_name().unwrap().to_string_lossy().to_string(),
                year: 2000,
                duration: String::from("YEYE"),
                // video: VideoStream {
                //     resolution: Resolution::from_str(&row.get::<_, String>(""))

                // }
                video: Resolution::from("720p"),
                hash,
                size: make_gb(size),
            }),
        }
    }
    //
    println!("EXISTING_HASHMAP: {:?}", existing);
    for z in collection {
        println!("{:?}", z);
    }
    //
    //
    //     for path in path_list {
    //         let (_, hash) = numov::read_metadata(&path);
    //
    //         if !existing.contains(&hash) {
    //             // Add movie to db
    //             println!("Adding movie to db...");
    //         } else {
    //             let mut stmt = db
    //                 .conn
    //                 .prepare("SELECT * FROM movies WHERE hash IN (?)")
    //                 .unwrap();
    //
    //             let mut rows = stmt.query([&hash]).unwrap();
    //
    //             // let rows = stmt
    //             //     .query_map([&hash], |r| {
    //             //         Ok(MiniMovie {
    //             //             title: r.get("title").unwrap(),
    //             //             year: r.get("year").unwrap(),
    //             //             hash: r.get("hash").unwrap(),
    //             //             size: r.get("size").unwrap(),
    //             //         })
    //             //     })
    //             //     .unwrap();
    //
    //             while let Some(row) = rows.next().unwrap() {
    //                 x.push(MiniMovie {
    //                     title: row.get("title").unwrap(),
    //                     year: row.get("year").unwrap(),
    //                     hash: row.get("hash").unwrap(),
    //                     size: row.get("size").unwrap(),
    //                 })
    //             }
    //         }
    //     }
    //     db.conn.execute("COMMIT", []).unwrap();
    //     println!("{:?}", x);
}

pub fn make_gb(bytes: u64) -> f32 {
    ["B", "KB", "MB", "GB"]
        .iter()
        .fold(bytes as f32, |acc, _| match acc > 1024.0 {
            true => acc / 1024.0,
            false => acc,
        })
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
