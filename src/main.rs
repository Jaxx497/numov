mod database;
mod library;
mod movie;
mod ratings;
mod temp;

use std::path::PathBuf;

use database::Database;
use library::Library;
use movie::Movie;
use ratings::Ratings;
use std::time::Instant;

fn main() {
    let dirs = Library::_get_dirs("M:/")[0..5].to_vec();

    for f in dirs {
        let movie = Movie::new(&f);
        println!("{}", movie);
    }
}

fn mainx() {
    let path_raw = r#"M:\Baby Driver (2017) [2160p x265 10bit AAC-7.1 Tigole] (11.71 GB)\Baby Driver (2017) (2160p BluRay x265 10bit HDR Tigole).mkv"#;
    let path_raw2 = r#"M:\12 Angry Men (1957) [1080p x265 10bit AAC-mono Silence] (3.43 GB)\12 Angry Men (1957) (1080p BluRay x265 Silence).mkv"#;
    let path_raw3 =
        r#"C:\Users\J\Desktop\locke\Locke (2013) (1080p BluRay x265 HEVC 10bit AAC 5.1 afm72).mkv"#;

    let path_buf = PathBuf::from(path_raw);
    let path_buf2 = PathBuf::from(path_raw2);
    let path_buf3 = PathBuf::from(path_raw3);

    temp::mat(&path_buf);
    temp::mat(&path_buf2);
    temp::mat(&path_buf3);

    //
    // println!("BABY DRIVER");
    //
    // for i in [&path_buf, &path_buf2] {
    // for i in [&path_buf, &path_buf2, &path_buf3] {
    //     let z = temp::mat(i);
    //     println!("\n{:?}", i);
    // }
    //
    // println!("FIN");
}

fn mainz() {
    let t1 = Instant::now();
    let db = match Database::new() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not make database!\nError: {e}");
            std::process::exit(1);
        }
    };

    // let x = Ratings::new(&db);
    // x.insert_to_table();

    let mut lib = Library::new("M:/", &db);
    lib.build().unwrap();

    lib.collection.iter().for_each(|x| println!("{}", x.title));

    db.conn.close().unwrap();
    println!("\nCompleted all tasks in {:.4?}", Instant::now() - t1);
}
