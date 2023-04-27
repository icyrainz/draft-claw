use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowListOptionAll, kCGWindowNumber, kCGWindowOwnerName,
};
use core_foundation::{
    array::CFArray,
    base::{CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::CFString,
};
use leptess::LepTess;
use std::process::Command;


const ETERNAL_WINDOW_NAME: &str = "Eternal Card Game";
const ETERNAL_SCREENSHOT_PATH: &str = "game.png";
const TESS_DATA: &str = "./resource/tessdata";

fn get_game_window_id() -> Option<u32> {
    let game_window = copy_window_info(
        kCGWindowListOptionAll,
        kCGNullWindowID
        ).unwrap()
        .get_all_values()
        .iter()
        .map(|&window_info| unsafe {
            let wininfo_hash: CFDictionary<CFString, CFType> =
                TCFType::wrap_under_get_rule(window_info as CFDictionaryRef);

            (
                wininfo_hash.get(kCGWindowOwnerName).downcast::<CFString>().unwrap().to_string(),
                wininfo_hash.get(kCGWindowNumber).downcast::<CFNumber>().unwrap().to_i32().unwrap()
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
        .arg(ETERNAL_SCREENSHOT_PATH)
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

fn capture_text_from_image(image_path: &str, with_data: bool) -> Result<String, String> {
    let tess_data = if with_data {
        Some(TESS_DATA)
    } else {
        None
    };
    
    let mut lt = LepTess::new(tess_data, "eng").expect("tesseract init failed");
    lt.set_image(image_path).expect("set image failed");

    println!("captured text: {}", lt.get_utf8_text().unwrap_or("no text".to_string()));
    lt.get_utf8_text().map_err(|err| err.to_string())
}

pub fn capture_loop() {
    match get_game_window_id() {
        None => {
            println!("game window not found");
        },
        Some(window_id) => {
            println!("found game window id: {}", window_id);
            capture_game_window(window_id).unwrap();
            match capture_text_from_image(ETERNAL_SCREENSHOT_PATH, false) {
                Ok(text) => {
                    println!("captured text: {}", text);
                },
                Err(err) => {
                    println!("capture text failed: {}", err);
                }
            }
        }
    }
}
