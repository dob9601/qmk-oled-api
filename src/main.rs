use std::env;
use std::error::Error;

use hidapi::HidApi;
use mpris::{Metadata, PlayerFinder};
use serde::{Serialize, Deserialize};

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
    let device_vendor_id: u16 = env::var("DEVICE_VENDOR_ID")
        .expect("Missing required env var")
        .parse()
        .expect("Could not parse vendor ID");
    let device_product_id: u16 = env::var("DEVICE_PRODUCT_ID")
        .expect("Missing required env var")
        .parse()
        .expect("Could not parse product ID");

    loop {
        let metadata = get_current_metadata()?;
        let hid_api = HidApi::new()?;
        let device = hid_api.open(device_vendor_id, device_product_id)?;
        device.write(&bincode::serialize(&metadata)?)?;
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
