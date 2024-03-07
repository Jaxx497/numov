mod database;
mod library;
mod movie;
mod ratings;
mod temp;

use std::path::PathBuf;

use database::Database;
use library::Library;
use ratings::Ratings;
use std::time::Instant;

fn mainz() {
    let path = r#"M:\Baby Driver (2017) [2160p x265 10bit AAC-7.1 Tigole] (11.71 GB)\Baby Driver (2017) (2160p BluRay x265 10bit HDR Tigole).mkv"#;
    let x = PathBuf::from(path);
    temp::mat(&x);
}

fn main() {
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
