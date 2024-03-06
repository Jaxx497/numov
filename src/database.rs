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
                        hash INTEGER NOT NULL UNIQUE, 
                        size REAL NOT NULL
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
