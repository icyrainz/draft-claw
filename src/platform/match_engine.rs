use std::error::Error;

use image::{DynamicImage, GrayImage, RgbImage};
use imageproc::rect::Rect;
use imageproc::template_matching::{match_template, MatchTemplateMethod};

fn find_card_locations(
    image_path: &str,
    template_path: &str,
    threshold: f32,
    ) -> Result<Vec<Rect>, Box<dyn Error>> {
    let image = image::open(image_path)?.to_luma8();
    let template = image::open(template_path)?.to_luma8();

    let method = MatchTemplateMethod::SumOfSquaredErrorsNormalized;
    let score_map = match_template(&image, &template, method);

    let mut card_locations = Vec::new();
    let template_width = template.width();
    let template_height = template.height();

    for y in 0..score_map.height() {
        for x in 0..score_map.width() {
            let score = score_map.get_pixel(x, y)[0];
            if score > threshold {
                let x = x as i32;
                let y = y as i32;
                let rect = Rect::at(x, y).of_size(template_width, template_height);
                card_locations.push(rect);
            }
        }
    }

    Ok(card_locations)
}
