mod database;
mod library;
mod movie;
mod movie_types;

use clap::Parser;
use clap::ValueEnum;
use library::Library;
use std::{path::PathBuf, time::Instant};

fn main() {
    let t1 = Instant::now();
    let args = Args::parse();

    let mut lib = Library::new("");

    if let Some(user) = args.lb_username {
        lib.update_ratings(&user)
            .unwrap_or_else(|e| println!("Error parsing ratings: {e}"));
    }

    match args.path {
        Some(root) if PathBuf::from(&root).is_dir() => {
            lib.root = root;
            lib.update_movies().ok();
            if args.rename {
                lib.rename_folders();
            }
        }
        _ => (),
    };

    if args.csv {
        lib.output_to_csv();
    }

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Updated database with movies from PATH
    #[arg(short, long)]
    path: Option<String>,

    /// Letterboxd user ratings to scrape
    #[arg(short, long = "letterboxd")]
    lb_username: Option<String>,

    /// Output movie data as a csv file
    #[arg(long, action = clap::ArgAction::SetTrue)]
    csv: bool,

    /// Rename folders
    #[arg(short = 'R', long = "rename", action = clap::ArgAction::SetTrue)]
    rename: bool,
    // /// Output movie data as a dataframe
    // #[arg(long = "df", action = clap::ArgAction::SetTrue)]
    // df: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum Output {
    Csv,
    Json,
    Df,
}

#[derive(Clone, Debug, ValueEnum)]
enum Update {
    M,
    Movies,
    R,
    Ratings,
    All,
}
