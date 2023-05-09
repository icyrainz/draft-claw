use super::{ocr_engine::*};

use core_foundation::{
    base::{CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::CFString,
};
use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowListOptionAll, kCGWindowNumber, kCGWindowOwnerName,
};
use lazy_static::lazy_static;
use leptess::LepTess;

use std::{fs, path::PathBuf, process::Command};

const RUNTIME_PATH: &str = "./";
const ETERNAL_WINDOW_NAME: &str = "Eternal Card Game";
const ETERNAL_SCREEN_FILE_NAME: &str = "game.png";
const ETERNAL_SCREEN_PROCESSED_FILE_NAME: &str = "game_processed.png";
const TESS_DATA: &str = "./resource/tessdata";

fn get_eternal_screen_path() -> Result<String, String> {
    fs::create_dir_all(RUNTIME_PATH).map_err(|err| err.to_string())?;

    let path = PathBuf::from(RUNTIME_PATH)
        .join(ETERNAL_SCREEN_FILE_NAME)
        .to_string_lossy()
        .to_string();
    Ok(path)
}

fn get_eternal_screen_processed_path() -> Result<String, String> {
    fs::create_dir_all(RUNTIME_PATH).map_err(|err| err.to_string())?;

    let path = PathBuf::from(RUNTIME_PATH)
        .join(ETERNAL_SCREEN_PROCESSED_FILE_NAME)
        .to_string_lossy()
        .to_string();
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

lazy_static! {
    pub static ref CARD_POSITIONS: Vec<ScreenRect> = vec![
        ScreenRect::new(508, 476, 237, 22),
        ScreenRect::new(838, 476, 237, 22),
        ScreenRect::new(1168, 476, 237, 22),
        ScreenRect::new(1500, 476, 237, 22),
        ScreenRect::new(508, 944, 237, 22),
        ScreenRect::new(838, 944, 237, 22),
        ScreenRect::new(1168, 944, 237, 22),
        ScreenRect::new(1500, 944, 237, 22),
        ScreenRect::new(508, 1411, 237, 22),
        ScreenRect::new(838, 1411, 237, 22),
        ScreenRect::new(1168, 1411, 237, 22),
        ScreenRect::new(1500, 1411, 237, 22),
    ];
    pub static ref DECK_POSITIONS: Vec<(ScreenRect, ScreenRect)> = vec![
        (ScreenRect::new(2194, 462, 269, 60), ScreenRect::new(2507, 469, 30, 50)),
        (ScreenRect::new(2194, 544, 269, 60), ScreenRect::new(2507, 544, 30, 50)),
        (ScreenRect::new(2194, 625, 269, 60), ScreenRect::new(2507, 625, 30, 50)),
        (ScreenRect::new(2194, 705, 269, 60), ScreenRect::new(2507, 705, 30, 50)),
        (ScreenRect::new(2194, 788, 269, 60), ScreenRect::new(2507, 788, 30, 50)),
        (ScreenRect::new(2194, 870, 269, 60), ScreenRect::new(2507, 870, 30, 50)),
        (ScreenRect::new(2194, 950, 269, 60), ScreenRect::new(2507, 950, 30, 50)),
        (ScreenRect::new(2194,1032, 269, 60), ScreenRect::new(2507,1032, 30, 50)),
        (ScreenRect::new(2194,1114, 269, 60), ScreenRect::new(2507,1114, 30, 50)),
        (ScreenRect::new(2194,1195, 269, 60), ScreenRect::new(2507,1195, 30, 50)),
        (ScreenRect::new(2194,1277, 269, 60), ScreenRect::new(2507,1277, 30, 50)),
        (ScreenRect::new(2194,1358, 269, 60), ScreenRect::new(2507,1358, 30, 50)),
        (ScreenRect::new(2194,1439, 269, 60), ScreenRect::new(2507,1439, 30, 50)),
        (ScreenRect::new(2194,1521, 269, 60), ScreenRect::new(2507,1521, 30, 50)),
        (ScreenRect::new(2194,1602, 269, 60), ScreenRect::new(2507,1602, 30, 50)),
    ];
    pub static ref PICK_NUM_POSITION: ScreenRect = ScreenRect::new(1504, 1601, 267, 48);
}

pub struct ScreenData {
    pub pick_num: String,
    pub cards: Vec<String>,
    pub deck: Vec<(String, String)>,
}

fn capture_raw_text_from_image(
    image_path: &str,
    with_data: bool,
) -> Result<ScreenData, String> {
    let tess_data = if with_data { Some(TESS_DATA) } else { None };

    let mut lt = LepTess::new(tess_data, "eng").expect("tesseract init failed");
    lt.set_image(image_path).expect("set image failed");

    let mut captured_card_vec = Vec::new();
    let mut captured_deck_vec = Vec::new();
    let (screen_width, screen_height) = lt.get_image_dimensions().unwrap();

    dbg!((screen_width, screen_height));

    for rect in CARD_POSITIONS.iter() {
        lt.set_rectangle(rect.x, rect.y, rect.width, rect.height);
        let text = lt.get_utf8_text().expect("get text failed");
        captured_card_vec.push(text);
    }
    dbg!(&captured_card_vec);

    lt.set_rectangle(
        PICK_NUM_POSITION.x,
        PICK_NUM_POSITION.y,
        PICK_NUM_POSITION.width,
        PICK_NUM_POSITION.height,
    );
    let pic_number_text = lt.get_utf8_text().expect("get text failed");

    for rect in DECK_POSITIONS.iter() {
        lt.set_rectangle(rect.0.x, rect.0.y, rect.0.width, rect.0.height);
        let text = lt.get_utf8_text().expect("get text failed");

        lt.set_rectangle(rect.1.x, rect.1.y, rect.1.width, rect.1.height);
        let count = lt.get_utf8_text().expect("get text failed");
        captured_deck_vec.push((text, count));
    }

    Ok(ScreenData {
        pick_num: pic_number_text,
        cards: captured_card_vec,
        deck: captured_deck_vec,
    })
}

pub fn capture_raw_text_on_screen() -> Result<ScreenData, String> {
    let screenshot_path = get_eternal_screen_path().unwrap();

    match get_game_window_id() {
        None => {
            return Err("game window not found".to_string());
        }
        Some(window_id) => {
            println!("found game window id: {}", window_id);
            capture_game_window(window_id).unwrap();
        }
    }

    let process_screenshot_path = get_eternal_screen_processed_path().unwrap();
    process_image(&screenshot_path, &process_screenshot_path).unwrap();

    capture_raw_text_from_image(&process_screenshot_path, true)
}

pub fn select_card(card_index: u8) -> Result<(), String> {
    let window_id = get_game_window_id().ok_or("game window not found")?;

    let (x, y) = (CARD_POSITIONS[card_index as usize].x, CARD_POSITIONS[card_index as usize].y);
    super::input::send_mouse_event(window_id, x as f64, y as f64);

    Ok(())
}
