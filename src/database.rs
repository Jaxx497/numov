use crate::movie::VideoStream;
use rusqlite::{Connection, Result};
use std::collections::HashMap;

use crate::{
    movie_types::{
        bitdepth::{self, BitDepth},
        resolution::Resolution,
        video_codec::VideoCodec,
    },
    temp,
};

#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("./numov_data.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS movies(
                        title TEXT NOT NULL,
                        year INTEGER NOT NULL,
                        size REAL NOT NULL,
                        duration TEXT NOT NULL,
                        resolution TEXT NOT NULL,
                        vid_codec TEXT NOT NULL, 
                        bit_depth TEXT NOT NULL,
                        aud_codec TEXT NOT NULL, 
                        channels NUMERIC NOT NULL,
                        aud_count INTEGER NOT NULL,
                        sub_format TEXT NOT NULL,
                        sub_count INTEGER NOT NULL,
                        hash INTEGER NOT NULL UNIQUE 
                    )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS ratings(
                        title TEXT PRIMARY KEY,
                        rating TEXT NOT NULL
                    )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn fetch_movies(&self) -> Result<HashMap<u32, temp::MiniMovie>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM movies")?;

        let existing = stmt
            .query_map([], |row| {
                Ok(temp::MiniMovie {
                    title: row.get("title")?,
                    year: row.get("year")?,
                    duration: row.get("duration")?,

                    video: row.get("resolution")?,
                    // video: VideoStream {
                    //     resolution: Resolution::from(&row.get::<&str, _>("resolution")?),
                    //     bit_depth: BitDepth::from(&row.get("bit_depth")?),
                    //     codec: VideoCodec::from(&row.get("vid_codec")?.to_string()),
                    // },
                    hash: row.get("hash")?,
                    size: row.get("size")?,
                })
            })?
            .filter_map(Result::ok)
            .map(|movie| (movie.hash, movie))
            .collect::<HashMap<u32, temp::MiniMovie>>();

        Ok(existing)
    }
}
