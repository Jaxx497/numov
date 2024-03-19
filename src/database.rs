use crate::movie::{AudioStream, Movie, SubtitleStream, VideoStream};
use rusqlite::{params, Connection, Result};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        
        let db_path = dirs::config_dir().unwrap().join("numov");
        if std::fs::metadata(&db_path).is_err() {
            std::fs::create_dir(&db_path).unwrap();
        }
        //
        let conn = Connection::open(db_path.join("data.db"))?;
        // let conn = Connection::open(db_path.join("data.db")).unwrap_or_else(|e| println!("{}", e));
        // let conn = Connection::open("numov.db")?;
        // let x = PathBuf::from(conn.path().unwrap());

        conn.execute(
            "CREATE TABLE IF NOT EXISTS movies(
                        title TEXT NOT NULL,
                        year INTEGER NOT NULL,
                        rating TEXT,
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
                    )", [],
        )?;

        Ok(Database { conn })
    }

    pub fn update_movie_table(&mut self, additions: &HashMap<u32, Movie>, removals: &HashSet<u32>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {

            let mut stmt = tx.prepare( 
                "INSERT OR REPLACE INTO movies (Title, Year, Rating, Size, Duration, Resolution, Vid_codec, Bit_depth, Aud_codec, Channels, Aud_count, Sub_format, Sub_count, Hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )?;

            for movie in additions.values() {
                stmt.execute( params![&movie.title,
                        &movie.year,
                        &movie.rating,
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
        {
            let mut stmt = tx.prepare("DELETE FROM movies WHERE hash = (?)")?;
            for hash in removals {
                stmt.execute(params![hash])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Provided a Hashmap of ratings, update the 'ratings' table
    pub fn update_ratings_table(&mut self, ratings_table: &HashMap<String, String>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO ratings (title, rating) VALUES (?, ?)"
            )?;

            for (k, v) in ratings_table {
                stmt.execute(params![k,v])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

// ===============
//   Fetch Data -> Fetch directly from database
// ===============
impl Database {
    pub fn fetch(&self) -> (HashMap<u32, Movie>, HashMap<String, String>) {
        let movies = self.fetch_movies().unwrap_or_else(|e| {
            println!("Could not fetch movies. Error: {e}"); 
            HashMap::new()
        });

        let ratings = self.fetch_ratings().unwrap_or_else(|e| {
            println!("Failed to fetch ratings. Error: {e}");
            HashMap::new()
        });
        (movies, ratings)
    }

    pub fn fetch_ratings(&self) -> Result<HashMap<String, String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT title, rating FROM ratings")?;
        let ratings = stmt.query_map([], 
            |row| Ok((row.get("title")?, row.get("rating")?)))?;

        ratings.collect()
    }

    pub fn fetch_movies(&self) -> Result<HashMap<u32, Movie>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM movies")?;

        let existing = stmt
            .query_map([], |row| {
                Ok(Movie {
                    title: row.get("title")?,
                    year: row.get("year")?,
                    rating: row.get("rating")?,
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
                    // path: PathBuf::new(),
                })
            })?
            .filter_map(Result::ok)
            .map(|movie| (movie.hash, movie))
            .collect::<HashMap<u32, Movie>>();

        Ok(existing)
    }
}

pub fn delete_db() {
    let db_path = dirs::config_dir().unwrap().join("numov/data.db");
        if std::fs::metadata(&db_path).is_ok() {
            match std::fs::remove_file(db_path) {
                Ok(_) => println!("Successfully deleted database!"),
                Err(e) => println!("Unable to delete database: {e}"),
        };
    } else {
        println!("Database file not found.");
    }
}
