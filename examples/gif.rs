use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::thread::sleep;

use image::codecs::gif::GifDecoder;
use image::imageops::{dither, BiLevel};
use image::{AnimationDecoder, DynamicImage};
use qmk_oled_api::screen::{OledScreen, ImageSizing};

fn main() -> Result<(), Box<dyn Error>> {
    let device_path =
        CString::new(env::var("DEVICE_PATH").expect("Missing required env var")).unwrap();

    let gif_file = File::open("examples/rick.gif")?;
    let gif_decoder = GifDecoder::new(gif_file)?;
    let frames = gif_decoder.into_frames().collect_frames()?;

    let mut screen = OledScreen::from_path(&device_path, 32, 128)?;

    loop {
        frames.iter().step_by(4).for_each(|frame| {
            let frame = frame;
            let image = frame.buffer().clone();
            let dynamic = DynamicImage::ImageRgba8(image);
            dither(&mut dynamic.grayscale().into_luma8(), &BiLevel);
            screen.draw_image(dynamic, -32, 0, &ImageSizing::Cover);
            screen.send().unwrap();
            sleep(frame.delay().into());
        });
    }
}
