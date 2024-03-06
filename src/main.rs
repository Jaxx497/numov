mod database;
mod library;
mod movie;
mod ratings;
use std::{alloc::System, process::ExitStatus};

use database::Database;
use library::Library;
use ratings::Ratings;

fn main() {
    let db = match Database::new() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        }
    };

    let x = Ratings::new(&db);
    x.insert_to_table();

    let mut lib = Library::new("M:/", &db);
    lib.build().unwrap();

    db.conn.close().unwrap();
    // println!("{:?}", lib);
    //
    // println!("{:?}", &lib);
    //
    // match lib.new.len() {
    //     n if n > 0 => println!("Added movies!"),
    //     _ => (),
    // }
    // for i in lib.new {
    //     println!("{}", i.hash);
    // }
    // match lib.existing.len() {
    //     n if n > 0 => println!("Removed movies!"),
    //     _ => (),
    // }
    // for i in lib.existing {
    //     println!("{}", i);
    // }
}
