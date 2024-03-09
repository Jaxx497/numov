use core::time::Duration;
use lazy_static::lazy_static;
use matroska::{
    self, Matroska,
    Settings::{Audio, Video},
    Track, Tracktype,
};
use regex::Regex;
use std::fmt;
use std::fmt::{Display, Formatter, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Movie {
    pub title: String,
    pub year: i16,
    pub duration: String,
    pub hash: u32,
    pub size: f32,
}

impl Movie {
    pub fn new(path: &PathBuf) -> Self {
        let matroska = Matroska::open(std::fs::File::open(path).unwrap()).unwrap();

        Self::collection(&matroska, path)
    }

    fn collection(matroska: &Matroska, path: &Path) -> Self {
        let (title, year) = Self::get_title_year(matroska, path).unwrap();
        println!("{title}");

        let (byte_count, hash) = numov::read_metadata(path);
        let duration = Self::readable_duration(&matroska.info.duration.unwrap());

        let size = numov::make_gb(byte_count);

        let vid_stream = Self::get_video_stream(&matroska.tracks);

        Movie {
            title,
            year,
            duration,
            hash,
            size,
        }
    }

    fn readable_duration(duration: &Duration) -> String {
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;

        format!("{}h {:02}min", hours, minutes)
    }

    fn get_title_year(matroska: &Matroska, path: &Path) -> Option<(String, i16)> {
        let metadata_title = matroska.info.title.clone().unwrap_or_default();

        let parent = path
            .parent()
            .expect("Could not unwrap parent contents.")
            .file_name()
            .expect("Could not read parent folder name.")
            .to_str()?;

        Self::extract_title_year(&metadata_title)
            .or_else(|| {
                Self::extract_title_year(parent).map(|(title, year)| {
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

    fn extract_title_year(str: &str) -> Option<(String, i16)> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<title>.*) \((?P<year>\d{4})\)").unwrap();
        }

        RE.captures(str).map(|captures| {
            let title = captures.get(1).unwrap().as_str().to_string();
            let year: i16 = captures.get(2).unwrap().as_str().parse().unwrap();
            (title, year)
        })
    }

    fn get_video_stream(tracks: &Vec<Track>) {
        // match &tracks[0].tracktype {
        //     Tracktype::Video => println!("VALID MKV"),
        //     _ => println!("THIS IS NOT GOOD"),
        // }

        for track in tracks {
            match track.tracktype {
                Tracktype::Subtitle => {
                    println!("{:?}", track.codec_id);
                }
                _ => {}
            }
        }
    }

    fn process_tracks(tracks: &[Track]) {
        let mut audio_codec;
        let mut audio channels: f32;
        let mut sub_format;
        let mut audio_count = 0;
        let mut sub_count = 0;

        for track in tracks {
            match track.tracktype {
                Tracktype::Audio => {
                    audio_count += 1;
                    if audio_count == 1 {
                        audio_codec = AudioCodec::from(track.codec_id.as_str());

                        if let Audio(x) = &track.settings {
                            println!("TODO MANAGE AUDIO CHANNELS")
                        }
                    }
                }
                Tracktype::Subtitle => {
                    sub_count += 1;
                    if sub_count == 1 {
                        sub_format = SubtitleFormat::from(track.codec_id.as_str());
                    }
                }
                _ => (),
            }
        }
        ()
    }
}

impl Display for Movie {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{0} ({1}) [{4:x}]\n\t {3} | {2:.2} GB\n",
            self.title, self.year, self.size, self.duration, self.hash
        )
    }
}



#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum AudioCodec {
    AAC,
    AC3,
    Atmos, // same as TrueHD
    EAC3,
    DTS,
    FLAC,
    OPUS,
    Other(String),
}

impl From<&str> for AudioCodec {
    fn from(s: &str) -> Self {
        match s {
            "A_AAC" => AudioCodec::AAC,
            "A_AC3" => AudioCodec::AC3,
            "A_DTS" => AudioCodec::DTS,
            "A_EAC3" => AudioCodec::EAC3,
            "A_FLAC" => AudioCodec::FLAC,
            "A_OPUS" => AudioCodec::OPUS,
            "A_TRUEHD" => AudioCodec::Atmos,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .next()
                    .unwrap_or("Err");
                AudioCodec::Other(other.to_string())
            }
        }
    }
}
impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioCodec::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum SubtitleFormat {
    ASS,
    PGS,
    SRT,
    SSA,
    VOB,
    Other(String),
}

impl From<&str> for SubtitleFormat {
    fn from(s: &str) -> Self {
        match s {
            "S_TEXT/ASS" => SubtitleFormat::ASS,
            "S_TEXT/PGS" => SubtitleFormat::PGS,
            "S_TEXT/UTF8" => SubtitleFormat::SRT,
            "S_TEXT/SSA" => SubtitleFormat::SSA,
            "S_VOBSUB" => SubtitleFormat::VOB,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .next()
                    .unwrap_or("Err");
                SubtitleFormat::Other(other.to_string())
            }
        }
    }
}
impl fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubtitleFormat::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}


#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum AudioChannels {
    1.0,
    2.0,
    4.0,
    5.1,
    
}

impl From<&str> for AudioChannels {
    fn from(s: &str) -> Self {
        match s {
            "A_AAC" => AudioChannels::AAC,
            "A_AC3" => AudioChannels::AC3,
            "A_DTS" => AudioChannels::DTS,
            "A_EAC3" => AudioChannels::EAC3,
            "A_FLAC" => AudioChannels::FLAC,
            "A_OPUS" => AudioChannels::OPUS,
            "A_TRUEHD" => AudioChannels::Atmos,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .next()
                    .unwrap_or("Err");
                AudioChannels::Other(other.to_string())
            }
        }
    }
}
impl fmt::Display for AudioChannels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioChannels::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}
