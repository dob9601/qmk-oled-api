use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::thread::sleep;

use image::codecs::gif::GifDecoder;
use image::imageops::{dither, BiLevel};
use image::{AnimationDecoder, DynamicImage};
use qmk_oled_api::screen::OledScreen32x128;

fn main() -> Result<(), Box<dyn Error>> {
    let device_path =
        CString::new(env::var("DEVICE_PATH").expect("Missing required env var")).unwrap();

    let gif_file = File::open("bird.gif")?;
    let gif_decoder = GifDecoder::new(gif_file)?; // FIXME: Replace with real path
    let frames = gif_decoder.into_frames().collect_frames()?;

    let mut screen = OledScreen32x128::from_path(&device_path)?;

    loop {
        frames.iter().for_each(|frame| {
            let frame = frame;
            let image = frame.buffer().clone();
            let dynamic = DynamicImage::ImageRgba8(image);
            dither(dynamic.grayscale().as_mut_luma8().unwrap(), &BiLevel);
            screen.draw_image(dynamic, 0, 0, true);
            sleep(frame.delay().into());
        });
    }
}
