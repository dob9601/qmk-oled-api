use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};

use hidapi::{HidApi, DeviceInfo};
use mpris::{Metadata, PlayerFinder};
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
    //let device_vendor_id: u16 = env::var("DEVICE_VENDOR_ID")
    //.expect("Missing required env var")
    //.parse()
    //.expect("Could not parse vendor ID");
    //let device_product_id: u16 = env::var("DEVICE_PRODUCT_ID")
    //.expect("Missing required env var")
    //.parse()
    //.expect("Could not parse product ID");
    let device_path = CString::new(env::var("DEVICE_PATH").expect("Missing required env var")).unwrap();

    let hid_api = HidApi::new().unwrap();
    loop {
        // let metadata = get_current_metadata()?;
        // println!("{:#?}", hid_api.device_list().collect::<Vec<&DeviceInfo>>());
        let device = hid_api.open_path(&device_path).unwrap();
        //println!("{:?}", device_info.path());
        //let device = device_info.open_device(&hid_api).unwrap();
        device.write("hi\n".as_bytes()).unwrap();
        std::thread::sleep_ms(1000);

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
