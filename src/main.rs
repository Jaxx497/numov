mod database;
mod library;
mod movie;
mod movie_types;
mod ratings;
mod temp;

use std::{fs::File, path::PathBuf};

use database::Database;
use library::Library;
use matroska::Matroska;
use movie::Movie;
use ratings::Ratings;
use std::time::Instant;

fn main() {
    let db = match Database::new() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        }
    };

    // let mut lib = Library::new("M:/", &db);
    // lib.build().unwrap();

    temp::build(&db);
}

fn mainx() {
    let t1 = Instant::now();
    let db = match Database::new() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        }
    };

    // let x = Ratings::new(&db);
    // x.insert_to_table();

    let mut lib = Library::new("M:/", &db);
    lib.build().unwrap();

    lib.collection.iter().for_each(|x| println!("{}", x.title));

    db.conn.close().unwrap();
    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
