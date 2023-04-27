mod card;
mod screen;
mod match_engine;

pub fn run() {
    println!("In platform!");

    screen::capture_loop();
    // card::load_card_data();
}
