use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::time::Duration;

use hidapi::{DeviceInfo, HidApi};
use mpris::{Metadata, PlayerFinder};
use qmk_nowplaying::screen::OledScreen32x128;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HIDSongMetadata {
    title: String,
    album: String,
    artist: String,
}

impl HIDSongMetadata {
    fn new(title: String, album: String, artist: String) -> Self {
        Self {
            title,
            album,
            artist,
        }
    }
}

impl From<mpris::Metadata> for HIDSongMetadata {
    fn from(metadata: mpris::Metadata) -> Self {
        HIDSongMetadata::new(
            metadata.title().unwrap_or("No Title").to_string(),
            metadata.album_name().unwrap_or("No Album").to_string(),
            metadata
                .album_artists()
                .map(|inner| inner.join(","))
                .unwrap_or_else(|| "No Artists".to_string()),
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let device_path =
        CString::new(env::var("DEVICE_PATH").expect("Missing required env var")).unwrap();

    let mut row = 0;

    let hid_api = HidApi::new().unwrap();
    let device = hid_api.open_path(&device_path).unwrap();
    loop {
        let mut screen = OledScreen32x128::new();
        screen.draw_image("/home/dob9601/Documents/Untitled.png", 0, 0);

        row += 1;
        if row == 32 { row = 0 }

        screen.send(&device)?;
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn get_current_metadata() -> Result<HIDSongMetadata, Box<dyn Error>> {
    let player_finder = PlayerFinder::new().map_err(|err| err.to_string())?;

    let players = player_finder.find_all().map_err(|err| err.to_string())?;

    let metadata: Option<Metadata> = players
        .iter()
        .map(|player| player.get_metadata())
        .filter_map(|metadata| metadata.ok())
        .find(|metadata| {
            if let Some(length) = metadata.length_in_microseconds() {
                length != 0
            } else {
                false
            }
        });

    if let Some(metadata) = metadata {
        println!("{metadata:#?}");
        Ok(metadata.into())
    } else {
        Err("No metadata found".into())
    }
}
