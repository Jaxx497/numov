mod database;
mod library;
mod movie;
mod movie_types;

use clap::{Parser, ValueEnum};
use library::Library;
use std::io::{self, Write};
use std::{env, path::PathBuf, time::Instant};

fn main() {
    let t1 = Instant::now();
    set_env();
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

            lib.update_movies();
            if args.rename {
                lib.rename_folders();
            }
        }
        _ => println!("Invalid path provided."),
    };

    if args.csv {
        lib.output_to_csv();
    }

    if let Some(x) = &args.dataframe {
        lib.handle_dataframe(x.as_str())
            .unwrap_or_else(|e| println!("Error creating dataframe: {e}"))
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
    #[arg(short = 'D', long, value_enum)]
    dataframe: Option<DFOpts>,
    //
    /// Reset database
    #[arg(long, action = clap::ArgAction::SetTrue)]
    reset: bool,
}

fn set_env() {
    env::set_var("POLARS_FMT_TABLE_FORMATTING", "UTF8_BORDERS_ONLY");
    env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    env::set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1");
    env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", "1");
    env::set_var("POLARS_FMT_MAX_ROWS", "25");
    env::set_var("POLARS_FMT_STR_LEN", "35");
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, ValueEnum)]
enum DFOpts {
    audio,
    channels,
    full,
    subs,
    year,
}

impl DFOpts {
    fn as_str(&self) -> &'static str {
        match self {
            DFOpts::audio => "audio",
            DFOpts::channels => "channels",
            DFOpts::full => "full",
            DFOpts::subs => "subs",
            DFOpts::year => "year",
        }
    }
}
