mod database;
mod library;
mod movie;
mod movie_types;

use strsim::jaro_winkler;

use library::Library;
use movie::Movie;
use std::time::Instant;

fn main() {
    let t1 = Instant::now();

    let mut lib = Library::new("M:/");

    // lib.update_movies()
    //     .unwrap_or_else(|e| println!("Error updating movies in database: {e}"));
    // lib.update_ratings().unwrap_or_default();
    //

    let mut x = Vec::new();
    for (_, v) in lib.existing {
        x.push(v);
    }

    for (title, rating) in lib.ratings {
        let mut best_match = (0.0, 0);

        for (index, movie) in x.iter().enumerate() {
            let similarity = jaro_winkler(&title.to_lowercase(), &movie.title.to_lowercase());
            if similarity > best_match.0 {
                best_match = (similarity, index);
            }
        }
        if best_match.0 >= 0.85 {
            println!("{} {}", x[best_match.1].title, title);
            x[best_match.1].rating = Some(rating);
        }
    }

    for i in x {
        println!("{} : {}", i.title, i.rating.unwrap_or_default());
    }

    // println!(
    //     "movies: {}\nratings: {}",
    //     lib.collection.len(),
    //     lib.ratings.len()
    // );

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
