mod database;
mod library;
mod movie;
mod movie_types;

use clap::Parser;
use clap::ValueEnum;
use library::Library;
use std::{path::PathBuf, time::Instant};

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    // let lib = if let Some(root) = args.root {
    //     if PathBuf::from(&root).is_dir() {
    //         Library::new(root)
    //     } else {
    //         eprintln!("Error: Invalid root directory specified.");
    //         std::process::exit(1);
    //     }
    // } else {
    //     eprintln!("Error: Invalid root directory specified.");
    //     std::process::exit(1);
    // };

    // let t1 = Instant::now();
    //
    // let mut lib = Library::new("M:/");
    //
    // lib.update_ratings("equus497").unwrap_or_default();
    // lib.update_movies();
    //
    // lib.rename_folders();
    //
    // println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Root directory of movie library
    #[arg(short, long)]
    root: Option<String>,

    /// Letterboxd User we will try to scrape
    #[arg(short, long = "letterboxd")]
    lb_user: Option<String>,

    /// Output data in a given format
    #[arg(short, long, value_enum)]
    output: Option<Output>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    csv: bool,
    #[arg(long,  action = clap::ArgAction::SetTrue)]
    json: bool,
    #[arg(long = "df", action = clap::ArgAction::SetTrue)]
    df: bool,
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
