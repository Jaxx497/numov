use matroska::Matroska;
use regex::Regex;
use std::path::PathBuf;

pub fn mat(path: &PathBuf) {
    let file = std::fs::File::open(path).unwrap();
    let matroska = Matroska::open(file).unwrap();

    // println!("{:#?}", matroska);
    //
    // let title2 = &matroska.info.title.unwrap();
    let title = match &matroska.info.title {
        Some(x) if is_proper(x) => String::from(x),
        _ => String::default(),
    };

    fn is_proper(s: &str) -> bool {
        Regex::new(r".(\(\d{4})$").unwrap().is_match(s)
    }

    println!("{:#?}", matroska);

    println!("{title}");
}
