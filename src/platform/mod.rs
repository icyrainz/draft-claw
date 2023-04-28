mod card;
mod screen;
mod match_engine;
mod db_access;
mod card_rating;

pub fn run() {
    println!("In platform!");

    // screen::capture_loop();
    // card::load_card_data();
    card_rating::load_card_rating();
}
