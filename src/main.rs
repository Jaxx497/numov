mod database;
mod library;
mod movie;
mod movie_types;

use library::Library;
use std::time::Instant;

fn main() {
    let t1 = Instant::now();

    let mut lib = Library::new("M:/");
    //
    lib.update_movies()
        .unwrap_or_else(|e| println!("Error updating movies in database: {e}"));
    // lib.update_ratings().unwrap_or_default();

    println!(
        "movies: {}\nratings: {}",
        lib.collection.len(),
        lib.ratings.len()
    );

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
