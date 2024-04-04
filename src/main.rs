mod database;
mod library;
mod movie;
mod movie_types;

use clap::Parser;
use library::Library;
use std::io::{self, Write};
use std::{path::PathBuf, time::Instant};

fn main() {
    let t1 = Instant::now();
    let args = Args::parse();

    if args.reset {
        print!("If you wish to completely reset the database, type \'KILL IT\' (without the quotes) » ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() && input.trim() == "KILL IT" {
            database::delete_db();
            std::process::exit(0);
        } else {
            println!("Database was not deleted. Exiting program.");
            std::process::exit(0);
        }
    }

    let mut lib = Library::new(PathBuf::new());

    if let Some(user) = &args.lb_username {
        lib.update_ratings(&user)
            .unwrap_or_else(|e| println!("Error parsing ratings: {e}"));
    }

    match &args.path {
        Some(root) if PathBuf::from(&root).is_dir() => {
            lib.root = PathBuf::from(&root)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(&root));

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

    if let Some(df_arg) = &args.dataframe {
        lib.handle_dataframe(df_arg)
            .unwrap_or_else(|e| println!("Error creating dataframe: {e}"));
    }

    // Getting library to close from within the library crate
    // has proven to be very difficult
    lib.db
        .conn
        .close()
        .unwrap_or_else(|(_, e)| println!("Could not close database: {e}"));

    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Path to read movies from
    #[arg(short = 'P', long)]
    path: Option<String>,

    /// Letterboxd user ratings to scrape
    #[arg(short = 'L', long = "letterboxd")]
    lb_username: Option<String>,

    /// Rename folders
    #[arg(short = 'R', long = "rename", action = clap::ArgAction::SetTrue)]
    rename: bool,

    /// Output movie data as a csv file
    #[arg(short = 'C', long, action = clap::ArgAction::SetTrue)]
    csv: bool,

    /// Output movie data as a dataframe
    // #[arg(short = 'D', long, default_value = "all")]
    #[arg(short = 'D', long)]
    dataframe: Option<String>,
    //
    /// Reset database
    #[arg(long, action = clap::ArgAction::SetTrue)]
    reset: bool,
}
