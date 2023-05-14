use lazy_static::lazy_static;
use leptess::LepTess;

use std::{fs::{self, File}, path::PathBuf, process::{Command, Stdio}};

const RUNTIME_PATH: &str = "./";
// const ETERNAL_WINDOW_NAME: &str = "Eternal Card Game";
const ETERNAL_WINDOW_NAME: &str = "Android Emulator";
const ETERNAL_SCREEN_FILE_NAME: &str = "game.png";
const ETERNAL_SCREEN_PROCESSED_FILE_NAME: &str = "game_processed.png";
const TESS_DATA: &str = "./resource/tessdata";

const ANDROID_ADB_PATH: &str = "/Users/tuephan/Library/Android/sdk/platform-tools/adb";

pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

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

fn capture_game_window_adb(output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path)
        .map_err(|err| format!("Failed to create output file: {}", err))?;

    let mut child = Command::new(ANDROID_ADB_PATH)
        .arg("exec-out")
        .arg("screencap")
        .arg("-p")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Capture game window from ADB failure: {}", err))?;

    if let Some(mut stdout) = child.stdout.take() {
        std::io::copy(&mut stdout, &mut file)
            .map_err(|err| format!("Failed to write screencap to output file: {}", err))?;
    }

    let status = child
        .wait()
        .map_err(|err| format!("Failed to wait for adb process: {}", err))?;

    if status.success() {
        Ok(())
    } else {
        Err("adb screencap failed".to_string())
    }
}
// 2560 x 1600
// lazy_static! {
//     pub static ref CARD_POSITIONS: Vec<ScreenRect> = vec![
//         ScreenRect::new( 438, 367, 239, 23),
//         ScreenRect::new( 769, 367, 239, 23),
//         ScreenRect::new(1100, 367, 239, 23),
//         ScreenRect::new(1430, 367, 239, 23),
//         ScreenRect::new( 438, 836, 239, 23),
//         ScreenRect::new( 769, 836, 239, 23),
//         ScreenRect::new(1100, 836, 239, 23),
//         ScreenRect::new(1430, 836, 239, 23),
//         ScreenRect::new( 438,1303, 239, 23),
//         ScreenRect::new( 769,1303, 239, 23),
//         ScreenRect::new(1100,1303, 239, 23),
//         ScreenRect::new(1430,1303, 239, 23),
//     ];
//     pub static ref DECK_POSITIONS: Vec<(ScreenRect, ScreenRect)> = vec![
//     ];
//     pub static ref PICK_NUM_POSITION: ScreenRect = ScreenRect::new(1419, 1495, 277, 43);
// }

// 1920 x 1080
lazy_static! {
    pub static ref CARD_POSITIONS: Vec<ScreenRect> = vec![
        ScreenRect::new( 393, 250, 160, 14),
        ScreenRect::new( 615, 250, 160, 14),
        ScreenRect::new( 839, 250, 160, 14),
        ScreenRect::new(1062, 250, 160, 14),
        ScreenRect::new( 393, 565, 160, 14),
        ScreenRect::new( 615, 565, 160, 14),
        ScreenRect::new( 839, 565, 160, 14),
        ScreenRect::new(1062, 565, 160, 14),
        ScreenRect::new( 393, 880, 160, 14),
        ScreenRect::new( 615, 880, 160, 14),
        ScreenRect::new( 839, 880, 160, 14),
        ScreenRect::new(1062, 880, 160, 14),
    ];
    pub static ref DECK_POSITIONS: Vec<(ScreenRect, ScreenRect)> = vec![
    ];
    pub static ref PICK_NUM_POSITION: ScreenRect = ScreenRect::new(1055, 1008, 185, 31);
}

pub struct ScreenData {
    pub pick_num: String,
    pub cards: Vec<String>,
    pub deck: Vec<(String, String)>,
}

fn capture_raw_text_from_image(image_path: &str, with_data: bool) -> Result<ScreenData, String> {
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
    dbg!(&pic_number_text);

    for rect in DECK_POSITIONS.iter() {
        lt.set_rectangle(rect.0.x, rect.0.y, rect.0.width, rect.0.height);
        let text = lt.get_utf8_text().expect("get text failed");

        lt.set_rectangle(rect.1.x, rect.1.y, rect.1.width, rect.1.height);
        let count = lt.get_utf8_text().expect("get text failed");
        captured_deck_vec.push((text, count));
    }
    dbg!(&captured_deck_vec);

    Ok(ScreenData {
        pick_num: pic_number_text,
        cards: captured_card_vec,
        deck: captured_deck_vec,
    })
}

pub fn capture_raw_text_on_screen() -> Result<ScreenData, String> {
    let screenshot_path = get_eternal_screen_path()?;
    let process_screenshot_path = get_eternal_screen_processed_path()?;

    capture_game_window_adb(&screenshot_path)?;

    super::ocr_engine::process_image(&screenshot_path, &process_screenshot_path)
        .map_err(|err| err.to_string())?;
    capture_raw_text_from_image(&process_screenshot_path, true)
}

fn get_card_position(card_index: u8) -> (i32, i32) {
    (
        CARD_POSITIONS[card_index as usize].x,
        CARD_POSITIONS[card_index as usize].y,
    )
}

pub fn select_card(card_index: u8) -> Result<(), String> {
    let (x, y) = get_card_position(card_index);

    Command::new(ANDROID_ADB_PATH)
        .arg("shell")
        .arg("input")
        .arg("tap")
        .arg(x.to_string())
        .arg(y.to_string())
        .output()
        .map_err(|err| err.to_string())
        .map(|out| {
            dbg!(&out);
        })
}
