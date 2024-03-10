use core::time::Duration;
use lazy_static::lazy_static;
use matroska::{
    self, Matroska,
    Settings::{Audio, Video},
    Track, Tracktype,
};
use regex::Regex;
use rusqlite::{types::ToSql, Result as RusqliteResult};
use std::fmt;
use std::fmt::{Display, Formatter, Result};
use std::path::{Path, PathBuf};

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<title>.*) \((?P<year>\d{4})\)").unwrap();
}

#[derive(Debug)]
pub struct VideoStream {
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub bit_depth: usize,
}

#[derive(Debug)]
pub struct AudioStream {
    pub codec: AudioCodec,
    pub channels: f32,
    pub count: usize,
}

#[derive(Debug)]
pub struct SubtitleStream {
    pub format: SubtitleFormat,
    pub count: usize,
}

#[derive(Debug)]
pub struct Movie {
    pub title: String,
    pub year: i16,
    pub duration: String,
    pub video: VideoStream,
    pub audio: AudioStream,
    pub subs: SubtitleStream,
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
        println!("{}", title);

        let (byte_count, hash) = numov::read_metadata(path);
        let duration = Self::readable_duration(&matroska.info.duration.unwrap());

        let size = numov::make_gb(byte_count);

        // let vid_stream = Self::get_video_stream(&matroska.tracks);
        let (audio, subs) = Self::process_tracks(&matroska.tracks);

        let video = Self::get_video_stream(&matroska.tracks[0]);

        Movie {
            title,
            year,
            duration,
            video,
            audio,
            subs,
            hash,
            size,
        }
    }
}

impl Movie {
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
        RE.captures(str).map(|captures| {
            let title = captures.get(1).unwrap().as_str().to_string();
            let year: i16 = captures.get(2).unwrap().as_str().parse().unwrap();
            (title, year)
        })
    }

    fn get_video_stream(track: &Track) -> VideoStream {
        let mut video_info = VideoStream {
            resolution: Resolution::SD,
            codec: VideoCodec::from("None"),
            bit_depth: 0,
        };
        match track.tracktype {
            Tracktype::Video => {
                if let Video(video) = &track.settings {
                    video_info.resolution = Resolution::from(video.pixel_height);
                }
                video_info.codec = VideoCodec::from(track.codec_id.as_str());
                video_info.bit_depth = match video_info.codec {
                    VideoCodec::x265 => 10,
                    VideoCodec::x264 => 8,
                    VideoCodec::Other(ref s) if s == "VP9" => 10,
                    _ => 0,
                };
            }
            _ => panic!(
                "Expected to read a video track for file. Instead found {:?}",
                track.name
            ),
        }
        video_info
    }

    fn process_tracks(tracks: &[Track]) -> (AudioStream, SubtitleStream) {
        let mut audio_info = AudioStream {
            codec: AudioCodec::from("NONE"),
            count: 0,
            channels: 0.0,
        };

        let mut sub_info = SubtitleStream {
            format: SubtitleFormat::from("NONE"),
            count: 0,
        };

        for track in tracks[1..].iter() {
            match track.tracktype {
                Tracktype::Audio => {
                    audio_info.count += 1;
                    if audio_info.count == 1 {
                        audio_info.codec = AudioCodec::from(track.codec_id.as_str());
                        if let Audio(audio) = &track.settings {
                            audio_info.channels = Self::map_audio_channels(audio.channels);
                        }
                    }
                }
                Tracktype::Subtitle => {
                    sub_info.count += 1;
                    if sub_info.count == 1 {
                        sub_info.format = SubtitleFormat::from(track.codec_id.as_str());
                    }
                }
                _ => (),
            }
        }
        (audio_info, sub_info)
    }

    fn map_audio_channels(channels: u64) -> f32 {
        match channels {
            0 => 1.0,
            2 => 2.0,
            4 => 4.0,
            6 => 5.1,
            7 => 6.1,
            8 => 8.1,
            _ => 0.0,
        }
    }
}

impl Display for Movie {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{} ({}) [{:x}]\n\t{} | {:.2} GB\n\tVideo: {} | {}\n\tAudio: {} | ({} tracks) | {}\n\tSubs:  {} ({} subs)\n",
            self.title,
            self.year,
            self.hash,
            self.duration,
            self.size,
            self.video.resolution,
            self.video.codec,
            self.audio.codec,
            self.audio.channels,
            self.audio.count,
            self.subs.format,
            self.subs.count
        )
    }
}

#[derive(Debug, Default)]
enum Resolution {
    SD,
    HD720,
    HD1080,
    UHD4K,
    UHD8K,
    #[default]
    Err,
}

impl From<u64> for Resolution {
    fn from(height: u64) -> Self {
        match height {
            h if h < 480 => Resolution::SD,
            h if h <= 720 => Resolution::HD720,
            h if h <= 1080 => Resolution::HD1080,
            h if h <= 2160 => Resolution::UHD4K,
            _ => Resolution::UHD8K,
        }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Resolution::SD => write!(f, "SD"),
            Resolution::HD720 => write!(f, "720p"),
            Resolution::HD1080 => write!(f, "1080p"),
            Resolution::UHD4K => write!(f, "4K"),
            _ => write!(f, "8K"),
        }
    }
}

#[derive(Debug)]
enum VideoBits {
    Bit10,
    Bit8,
    Other(i8),
}

impl fmt::Display for VideoBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            VideoBits::Bit10 => write!(f, "10bit"),
            VideoBits::Bit8 => write!(f, "8bit"),
            _ => write!(f, "ERR"),
        }
    }
}

impl From<&i8> for VideoBits {
    fn from(bits: &i8) -> Self {
        match bits {
            10 => VideoBits::Bit10,
            8 => VideoBits::Bit8,
            i => VideoBits::Other(*i),
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum VideoCodec {
    x264,
    x265,
    Other(String),
}

impl From<&str> for VideoCodec {
    fn from(s: &str) -> Self {
        match s {
            "V_MPEGH/ISO/HEVC" => VideoCodec::x265,
            "V_MPEG4/ISO/AVC" => VideoCodec::x264,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .last()
                    .unwrap_or("Err");
                VideoCodec::Other(other.to_string())
            }
        }
    }
}

impl ToSql for VideoCodec {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            VideoCodec::x265 => rusqlite::types::ToSqlOutput::from("x265"),
            VideoCodec::x264 => rusqlite::types::ToSqlOutput::from("x264"),
            VideoCodec::Other(s) => rusqlite::types::ToSqlOutput::from(s.as_str()),
        })
    }
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoCodec::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
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
            "S_HDMV/PGS" => SubtitleFormat::PGS,
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
