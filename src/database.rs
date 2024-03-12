use crate::movie::{AudioStream, Movie, SubtitleStream, VideoStream};
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

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
                        hash INTEGER NOT NULL PRIMARY KEY 
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

    pub fn fetch_movies(&self) -> Result<HashMap<u32, Movie>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM movies")?;

        let existing = stmt
            .query_map([], |row| {
                Ok(Movie {
                    title: row.get("title")?,
                    year: row.get("year")?,
                    duration: row.get("duration")?,
                    video: VideoStream {
                        resolution: row.get("resolution")?,
                        codec: row.get("vid_codec")?,
                        bit_depth: row.get("bit_depth")?,
                    },
                    audio: AudioStream {
                        codec: row.get("aud_codec")?,
                        channels: row.get("channels")?,
                        count: row.get("aud_count")?,
                    },
                    subs: SubtitleStream {
                        format: row.get("sub_format")?,
                        count: row.get("sub_count")?,
                    },
                    hash: row.get("hash")?,
                    size: row.get("size")?,
                })
            })?
            .filter_map(Result::ok)
            .map(|movie| (movie.hash, movie))
            .collect::<HashMap<u32, Movie>>();

        Ok(existing)
    }

    pub fn bulk_insert(&mut self, new_movies: &Vec<Movie>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {

            let mut stmt = tx.prepare( 
                "INSERT INTO movies (Title, Year, Size, Duration, Resolution, Vid_codec, Bit_depth, Aud_codec, Channels, Aud_count, Sub_format, Sub_count, Hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )",
            )?;

            for movie in new_movies {
                stmt.execute( params![&movie.title,
                        &movie.year,
                        format!("{:.2}", &movie.size),
                        &movie.duration,
                        &movie.video.resolution,
                        &movie.video.codec,
                        &movie.video.bit_depth,
                        &movie.audio.codec,
                        &movie.audio.channels.to_string(),
                        &movie.audio.count,
                        &movie.subs.format,
                        &movie.subs.count,
                        &movie.hash]
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn bulk_removal(&mut self, old_hashes: &HashMap<u32, Movie>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM movies WHERE hash = (?)")?;

            for hash in old_hashes.keys() {
                stmt.execute(params![hash])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
