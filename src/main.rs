mod database;
mod library;
mod movie;
mod movie_types;

use library::Library;
use std::time::Instant;

fn main() {
    let t1 = Instant::now();

    let mut lib = Library::new("M:/");

    // lib.update_ratings("equus497").unwrap_or_default();
    lib.update_movies();

    lib.rename_folders();

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
