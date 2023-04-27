use super::match_engine::*;

use core_foundation::{
    array::CFArray,
    base::{CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::CFString,
};
use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowListOptionAll, kCGWindowNumber, kCGWindowOwnerName,
};
use leptess::LepTess;
use image::io::Reader as ImageReader;
use image::DynamicImage;

use std::{fs, process::Command, path::PathBuf};

const RUNTIME_PATH: &str = "./";
const ETERNAL_WINDOW_NAME: &str = "Eternal Card Game";
const ETERNAL_SCREEN_FILE_NAME: &str = "game.png";
const ETERNAL_SCREEN_PROCESSED_FILE_NAME: &str = "game_processed.png";
const TESS_DATA: &str = "./resource/tessdata";

fn get_eternal_screen_path() -> Result<String, String> {
    fs::create_dir_all(RUNTIME_PATH).map_err(|err| err.to_string())?;

    let path = PathBuf::from(RUNTIME_PATH).join(ETERNAL_SCREEN_FILE_NAME).to_string_lossy().to_string();
    Ok(path)
}

fn get_eternal_screen_processed_path() -> Result<String, String> {
    fs::create_dir_all(RUNTIME_PATH).map_err(|err| err.to_string())?;

    let path = PathBuf::from(RUNTIME_PATH).join(ETERNAL_SCREEN_PROCESSED_FILE_NAME).to_string_lossy().to_string();
    Ok(path)
}

fn get_game_window_id() -> Option<u32> {
    let game_window = copy_window_info(kCGWindowListOptionAll, kCGNullWindowID)
        .unwrap()
        .get_all_values()
        .iter()
        .map(|&window_info| unsafe {
            let wininfo_hash: CFDictionary<CFString, CFType> =
                TCFType::wrap_under_get_rule(window_info as CFDictionaryRef);

            (
                wininfo_hash
                    .get(kCGWindowOwnerName)
                    .downcast::<CFString>()
                    .unwrap()
                    .to_string(),
                wininfo_hash
                    .get(kCGWindowNumber)
                    .downcast::<CFNumber>()
                    .unwrap()
                    .to_i32()
                    .unwrap(),
            )
        })
        .find(|(window_name, _)| window_name == ETERNAL_WINDOW_NAME)
        .map(|(_, window_number)| window_number as u32);

    game_window
}

fn capture_game_window(window_id: u32) -> Result<(), String> {
    Command::new("screencapture")
        .arg("-l")
        .arg(window_id.to_string())
        .arg("-x")
        .arg(PathBuf::from(RUNTIME_PATH).join(ETERNAL_SCREEN_FILE_NAME))
        .status()
        .map_err(|err| err.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("screencapture failed".to_string())
            }
        })
}

fn capture_text_from_image(
    image_path: &str,
    rectangles: &[ScreenRect],
    with_data: bool
) -> Result<Vec<String>, String> {
    let tess_data = if with_data { Some(TESS_DATA) } else { None };

    let mut lt = LepTess::new(tess_data, "eng").expect("tesseract init failed");
    lt.set_image(image_path).expect("set image failed");

    let mut captured_text = Vec::new();

    for rect in rectangles {
        lt.set_rectangle(rect.x, rect.y, rect.width, rect.height);
        let text = lt.get_utf8_text().expect("get text failed");
        captured_text.push(text);
    }

    Ok(captured_text)
}


pub fn capture_loop() {
    let screenshot_path = get_eternal_screen_path().unwrap();

    match get_game_window_id() {
        None => {
            println!("game window not found");
        }
        Some(window_id) => {
            println!("found game window id: {}", window_id);
            capture_game_window(window_id).unwrap();
        }
    }

    let process_screenshot_path = get_eternal_screen_processed_path().unwrap();
    process_image(&screenshot_path, &process_screenshot_path).unwrap();

    let rectangles = vec![
        ScreenRect {
            x: 496,
            y: 466,
            width: 1230,
            height: 41,
        },
    ];

    match capture_text_from_image(&process_screenshot_path, &rectangles, false) {
        Ok(text) => {
            dbg!(&text);
        }
        Err(err) => {
            println!("capture text failed: {}", err);
        }
    }
}
