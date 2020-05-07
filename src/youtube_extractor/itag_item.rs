use lazy_static::lazy_static;

use ItagType::*;

lazy_static! {
    static ref ITAG_LIST: Vec<Itag> = vec![
        Itag {
            id: 17,
            itag_type: Video,
            resolution_string: String::from("240p"),
            ..Itag::default()
        },
        Itag {
            id: 36,
            itag_type: Video,
            resolution_string: String::from("240p"),
            ..Itag::default()
        },
        Itag {
            id: 18,
            itag_type: Video,
            resolution_string: String::from("360p"),
            ..Itag::default()
        },
        Itag {
            id: 34,
            itag_type: Video,
            resolution_string: String::from("360p"),
            ..Itag::default()
        },
        Itag {
            id: 35,
            itag_type: Video,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 59,
            itag_type: Video,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 78,
            itag_type: Video,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 22,
            itag_type: Video,
            resolution_string: String::from("720p"),
            ..Itag::default()
        },
        Itag {
            id: 37,
            itag_type: Video,
            resolution_string: String::from("1080p"),
            ..Itag::default()
        },
        Itag {
            id: 38,
            itag_type: Video,
            resolution_string: String::from("1080p"),
            ..Itag::default()
        },
        Itag {
            id: 43,
            itag_type: Video,
            resolution_string: String::from("360p"),
            ..Itag::default()
        },
        Itag {
            id: 44,
            itag_type: Video,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 45,
            itag_type: Video,
            resolution_string: String::from("720p"),
            ..Itag::default()
        },
        Itag {
            id: 46,
            itag_type: Video,
            resolution_string: String::from("1080p"),
            ..Itag::default()
        },
        Itag {
            id: 171,
            itag_type: Audio,
            avg_bitrate: 128,
            ..Itag::default()
        },
        Itag {
            id: 172,
            itag_type: Audio,
            avg_bitrate: 256,
            ..Itag::default()
        },
        Itag {
            id: 139,
            itag_type: Audio,
            avg_bitrate: 48,
            ..Itag::default()
        },
        Itag {
            id: 140,
            itag_type: Audio,
            avg_bitrate: 128,
            ..Itag::default()
        },
        Itag {
            id: 141,
            itag_type: Audio,
            avg_bitrate: 256,
            ..Itag::default()
        },
        Itag {
            id: 249,
            itag_type: Audio,
            avg_bitrate: 50,
            ..Itag::default()
        },
        Itag {
            id: 250,
            itag_type: Audio,
            avg_bitrate: 70,
            ..Itag::default()
        },
        Itag {
            id: 251,
            itag_type: Audio,
            avg_bitrate: 160,
            ..Itag::default()
        },
        Itag {
            id: 160,
            itag_type: VideoOnly,
            resolution_string: String::from("144p"),
            ..Itag::default()
        },
        Itag {
            id: 133,
            itag_type: VideoOnly,
            resolution_string: String::from("240p"),
            ..Itag::default()
        },
        Itag {
            id: 135,
            itag_type: VideoOnly,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 212,
            itag_type: VideoOnly,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 298,
            itag_type: VideoOnly,
            resolution_string: String::from("720p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 137,
            itag_type: VideoOnly,
            resolution_string: String::from("1080p"),
            ..Itag::default()
        },
        Itag {
            id: 299,
            itag_type: VideoOnly,
            resolution_string: String::from("1080p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 266,
            itag_type: VideoOnly,
            resolution_string: String::from("2160p"),
            ..Itag::default()
        },
        Itag {
            id: 278,
            itag_type: VideoOnly,
            resolution_string: String::from("144p"),
            ..Itag::default()
        },
        Itag {
            id: 242,
            itag_type: VideoOnly,
            resolution_string: String::from("240p"),
            ..Itag::default()
        },
        Itag {
            id: 244,
            itag_type: VideoOnly,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 245,
            itag_type: VideoOnly,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 246,
            itag_type: VideoOnly,
            resolution_string: String::from("480p"),
            ..Itag::default()
        },
        Itag {
            id: 247,
            itag_type: VideoOnly,
            resolution_string: String::from("720p"),
            ..Itag::default()
        },
        Itag {
            id: 248,
            itag_type: VideoOnly,
            resolution_string: String::from("1080p"),
            ..Itag::default()
        },
        Itag {
            id: 271,
            itag_type: VideoOnly,
            resolution_string: String::from("1440p"),
            ..Itag::default()
        },
        Itag {
            id: 272,
            itag_type: VideoOnly,
            resolution_string: String::from("2160p"),
            ..Itag::default()
        },
        Itag {
            id: 302,
            itag_type: VideoOnly,
            resolution_string: String::from("720p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 303,
            itag_type: VideoOnly,
            resolution_string: String::from("1080p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 308,
            itag_type: VideoOnly,
            resolution_string: String::from("1440p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 313,
            itag_type: VideoOnly,
            resolution_string: String::from("2160p"),
            ..Itag::default()
        },
        Itag {
            id: 315,
            itag_type: VideoOnly,
            resolution_string: String::from("2160p60"),
            fps: 60,
            ..Itag::default()
        },
        Itag {
            id: 394,
            fps: 24,
            resolution_string: String::from("144p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
        Itag {
            id: 395,
            resolution_string: String::from("240p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
        Itag {
            id: 396,
            resolution_string: String::from("360p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
        Itag {
            id: 397,
            resolution_string: String::from("480p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
        Itag {
            id: 398,
            resolution_string: String::from("720p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
        Itag {
            id: 399,
            resolution_string: String::from("1080p AV1"),
            itag_type: VideoOnly,
            ..Itag::default()
        },
    ];
}

#[derive(PartialEq, Clone, Debug)]
pub enum ItagType {
    Audio,
    Video,
    VideoOnly,
}

impl Default for ItagType {
    fn default() -> Self {
        Video
    }
}

#[derive(Default, Clone, Debug)]
pub struct Itag {
    pub id: i64,
    pub itag_type: ItagType,
    pub avg_bitrate: i64,
    pub resolution_string: String,
    pub fps: i64,
}

impl Itag {
    pub fn is_supported(id: i64) -> bool {
        for itag in ITAG_LIST.iter() {
            if itag.id == id {
                return true;
            }
        }
        false
    }

    pub fn get_itag(id: i64) -> Result<Itag, String> {
        for itag in ITAG_LIST.iter() {
            if itag.id == id {
                return Ok(itag.clone());
            }
        }
        Err(format!("Itag id {} not found", id))
    }
}
