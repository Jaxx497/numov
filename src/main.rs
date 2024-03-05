mod database;
mod library;
mod movie;
use database::Database;
use library::Library;

fn main() {
    let db = Database::new().unwrap();
    let mut lib = Library::new("M:/", &db);
    lib.build().unwrap();

    println!("{:?}", &lib);

    match lib.new.len() {
        n if n > 0 => println!("Added movies!"),
        _ => (),
    }
    for i in lib.new {
        println!("{}", i.hash);
    }
    match lib.existing.len() {
        n if n > 0 => println!("Removed movies!"),
        _ => (),
    }
    for i in lib.existing {
        println!("{}", i);
    }
}
