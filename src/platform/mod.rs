mod card;
mod match_engine;
mod screen;

pub fn run() {
    println!("In platform!");

    screen::capture_loop();
    // card::load_card_data();
}
