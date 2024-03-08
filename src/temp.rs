use lazy_static::lazy_static;
use matroska::Matroska;
use regex::Regex;
use std::path::{Path, PathBuf};

pub fn mat(path: &PathBuf) -> Option<()> {
    let file = std::fs::File::open(path).unwrap();
    let matroska = Matroska::open(file).unwrap();

    let (title, year) = get_title_year(&matroska, path)?;
    let (size, hash) = numov::read_metadata(&path);

    let duration_raw = &matroska.info.duration?;
    let hours = duration_raw.as_secs() / 3600;
    let minutes = (duration_raw.as_secs() % 3600) / 60;

    let duration = format!("{}h {:02}min", hours, minutes);

    // let (rating, duration_raw, video, audio, subs, encoder) = ;

    println!("{} ({}) | Duration: {}", title, year, duration);

    Some(())
}

/// Get title and year fields
///
/// # Params
/// `matroska: &Matroksa` Reference to a valid Matroska type
/// `path: &PathBuf` Reference to a valid path
///
/// Extract title and year from metadata title field
/// If that fails, try extracting from the parent folder
/// If success => Try to update the metadata via mkvprop
/// Else => Print out error statement, return NONE
pub fn get_title_year(matroska: &Matroska, path: &Path) -> Option<(String, i16)> {
    let metadata_title = matroska.info.title.clone().unwrap_or_default();

    let parent = path
        .parent()
        .expect("Could not unwrap parent contents.")
        .file_name()
        .expect("Could not read parent folder name.")
        .to_str()?;

    extract(&metadata_title)
        .or_else(|| {
            extract(parent).map(|(title, year)| {
                println!("TODO UPDATE METADATA FOR {}", title);
                (title, year)
            })
        })
        .or_else(|| {
            println!(
                "UNABLE TO PARSE TITLE INFO FOR {{ {:?} }}",
                &path.file_name().unwrap()
            );
            None
        })
}

fn extract(str: &str) -> Option<(String, i16)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<title>.*) \((?P<year>\d{4})\)").unwrap();
    }

    RE.captures(str).map(|captures| {
        let title = captures.get(1).unwrap().as_str().to_string();
        let year: i16 = captures.get(2).unwrap().as_str().parse().unwrap();
        (title, year)
    })
}

// If metadata contains a title field
//      And if title field matches `title name (year)` format
//          extract `title name` and `year` to respective variables
// Else if metadata does not contain a title field
//      read in path of file
//      From root_dir/parent_dir/title name (year) other text
//          extract `title name` and `year` to respective variables
//
//
// match extract(&metadata_title) {
//     Some(x) => Some(x),
//     None => match extract(parent) {
//         Some(x) => {
//             println!("TODO UPDATE METADATA");
//             Some(x)
//         }
//         None => {
//             println!(
//                 "UNABLE TO PARSE TITLE INFO FOR {{ {:?} }}",
//                 &path.file_name().unwrap()
//             );
//             None
//         }
//     },
// }

// .or_else(|| {
//     Command::new("mkvpropedit")
//         .spawn()
//         .ok()
//         .and_then(|_| get_user_input())
// })
