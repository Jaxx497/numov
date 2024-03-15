use crate::movie_types::{
    audio_codec::AudioCodec, bitdepth::BitDepth, resolution::Resolution,
    sub_format::SubtitleFormat, video_codec::VideoCodec,
};
use core::time::Duration;
use lazy_static::lazy_static;
use matroska::{
    self, Matroska,
    Settings::{Audio, Video},
    Track, Tracktype,
};
use regex::Regex;
use serde::Serialize;
use std::fmt::{Display, Formatter, Result};
use std::path::{Path, PathBuf};
use xxhash_rust::const_xxh32::xxh32;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<title>.*) \((?P<year>\d{4})\)").unwrap();
}

#[derive(Debug, Serialize)]
pub struct VideoStream {
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub bit_depth: BitDepth,
}

#[derive(Debug, Serialize)]
pub struct AudioStream {
    pub codec: AudioCodec,
    pub channels: f32,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct SubtitleStream {
    pub format: SubtitleFormat,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct Movie {
    pub title: String,
    pub year: i16,
    pub rating: Option<String>,
    pub size: f32,
    pub duration: String,
    pub video: VideoStream,
    pub audio: AudioStream,
    pub subs: SubtitleStream,
    pub hash: u32,
    pub path: PathBuf,
}

impl Movie {
    pub fn new(path: &PathBuf) -> Self {
        let matroska = Matroska::open(std::fs::File::open(path).unwrap()).unwrap();
        Self::collect(&matroska, path)
    }

    fn collect(matroska: &Matroska, path: &Path) -> Self {
        let (title, year) = Self::get_title_year(matroska, path).unwrap();
        let (byte_count, hash) = Self::read_metadata(path);
        let duration = Self::readable_duration(&matroska.info.duration.unwrap());
        let size = Self::make_gb(byte_count);
        let video = Self::get_video_stream(&matroska.tracks[0]);
        let (audio, subs) = Self::process_tracks(&matroska.tracks);

        Movie {
            title,
            year,
            rating: None,
            duration,
            video,
            audio,
            subs,
            hash,
            size,
            path: path.to_owned(),
        }
    }

    fn readable_duration(duration: &Duration) -> String {
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;

        format!("{}h {:02}min", hours, minutes)
    }

    pub fn make_gb(bytes: u64) -> f32 {
        ["B", "KB", "MB", "GB"]
            .iter()
            .fold(bytes as f32, |acc, _| match acc > 1024.0 {
                true => acc / 1024.0,
                false => acc,
            })
    }

    pub fn read_metadata(path: &Path) -> (u64, u32) {
        let metadata = std::fs::metadata(path).expect("Could not read the files metadata.");
        let bytes = metadata.len();
        let last_mod = metadata
            .modified()
            .expect("Could not obtain last modified date of file.")
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Could not convert to timestamp.")
            .as_nanos();

        let hash_input = format!("{}{}{}", bytes, last_mod, &path.display());
        (bytes, xxh32(hash_input.as_bytes(), 0))
    }

    fn get_title_year(matroska: &Matroska, path: &Path) -> Option<(String, i16)> {
        let metadata_title = matroska.info.title.clone().unwrap_or_default();

        let parent = path
            .parent()
            .expect("Could not unwrap parent contents.")
            .file_name()
            .expect("Could not read parent folder name.")
            .to_str()?;

        Self::extract_title_year(metadata_title)
            .or_else(|| {
                Self::extract_title_year(parent).map(|(title, year)| {
                    Self::mkvinfo_update(&title, year, path);
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

    fn extract_title_year(str: impl AsRef<str>) -> Option<(String, i16)> {
        RE.captures(str.as_ref()).map(|captures| {
            let title = captures.get(1).unwrap().as_str().to_string();
            let year: i16 = captures.get(2).unwrap().as_str().parse().unwrap();
            (title, year)
        })
    }

    fn mkvinfo_update(title: impl AsRef<str>, year: i16, path: &Path) {
        let formatted_title = format!("title={} ({year})", title.as_ref());

        let output = std::process::Command::new("mkvpropedit")
            .arg(path.to_str().unwrap())
            .arg("--tags")
            .arg("all:")
            .arg("--edit")
            .arg("info")
            .arg("--set")
            .arg(&formatted_title)
            .arg("--edit")
            .arg("track:s1")
            .arg("--set")
            .arg("flag-default=1")
            .output();

        match output {
            Ok(o) => match o.status.success() {
                true => println!(
                    "Wrote title to metadata of file. [{}]",
                    &formatted_title.split('=').last().unwrap_or_default()
                ),
                false => println!("FAILED TO UPDATE FILE TITLE: {:?}", path.file_name()),
            },
            Err(e) => println!("Failed to run mkvpropedit with error: {e}"),
        }
    }

    fn get_video_stream(track: &Track) -> VideoStream {
        let codec = VideoCodec::from(track.codec_id.as_str());
        let bit_depth = match codec {
            VideoCodec::x265 => BitDepth::Bit10,
            VideoCodec::x264 => BitDepth::Bit8,
            VideoCodec::Other(ref s) if s == "VP9" => BitDepth::Bit10,
            _ => BitDepth::Other(0),
        };
        let resolution = if let Video(video) = &track.settings {
            Resolution::from(video.pixel_height)
        } else {
            Resolution::Err
        };
        VideoStream {
            resolution,
            codec,
            bit_depth,
        }
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
            8 => 7.1,
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
