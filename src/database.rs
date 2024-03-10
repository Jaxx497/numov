use rusqlite::Connection;
use rusqlite::Result;

#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("./movies.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS movies(
                        title TEXT NOT NULL,
                        year INTEGER NOT NULL,
                        size REAL NOT NULL
                        duration TEXT NOT NULL,
                        resolution TEXT NOT NULL,
                        v_codec TEXT NOT NULL UNIQUE, 
                        bit_depth INTEGER NOT NULL
                        a_codec TEXT NOT NULL UNIQUE, 
                        channels REAL NOT NULL
                        a_count INTEGER NOT NULL
                        sub_format TEXT NOT NULL
                        sub_count INTEGER NOT NULL
                        hash INTEGER NOT NULL UNIQUE, 
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
}
