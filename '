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
    match lib.update_movies() {
        Ok((new, removed)) => {
            if new > 0 {
                println!("Added {new} movies!");
            }
            if removed > 0 {
                println!("Removed {removed} movies!");
            }
        }
        Err(e) => println!("Failed to update movies.\nError: {e}"),
    }

    lib.rename_folders();

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
