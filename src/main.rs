mod database;
mod library;
mod movie;
mod movie_types;
mod ratings;
mod temp;

use database::Database;
use library::Library;
use std::time::Instant;

fn main() {
    let t1 = Instant::now();
    let mut lib = Library::new("M:/");
    lib.build().unwrap();

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
