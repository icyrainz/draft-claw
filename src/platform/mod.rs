mod screen;
mod card;

pub fn run() {
    println!("In platform!");

    screen::capture_loop();
    // card::load_card_data();
}
