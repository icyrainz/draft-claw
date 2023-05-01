use std::error::Error;

use opencv::prelude::*;

pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub fn find_card_locations(
    image_path: &str,
    template_path: &str,
    threshold: f32,
) -> Result<Vec<ScreenRect>, Box<dyn Error>> {

    let matching_method = "opencv_edge";
    if matching_method == "opencv_edge" {
        find_card_locations_opencv_edge(image_path) 
    } else {
        find_card_locations_imageproc(image_path, template_path, threshold)
    }
}

pub fn process_image(image_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let src = opencv::imgcodecs::imread(
        image_path,
        opencv::imgcodecs::IMREAD_COLOR)
        .map_err(|err| Box::new(err) as Box<dyn Error>)?;
    let mut gray_src = Mat::default();
    opencv::imgproc::cvt_color(
        &src,
        &mut gray_src,
        opencv::imgproc::COLOR_BGR2GRAY,
        0
    )?;
    let result = gray_src.clone();

    // let mut blurred = Mat::default();
    // opencv::imgproc::gaussian_blur(
    //     &result,
    //     &mut blurred,
    //     opencv::core::Size::new(5, 5),
    //     0.0,
    //     0.0,
    //     opencv::core::BORDER_DEFAULT,
    // )?;
    // let result = blurred.clone();

    // let mut thresholded = Mat::default();
    // opencv::imgproc::adaptive_threshold(
    //     &result,
    //     &mut thresholded,
    //     255.0,
    //     opencv::imgproc::ADAPTIVE_THRESH_GAUSSIAN_C,
    //     opencv::imgproc::THRESH_BINARY,
    //     11,
    //     2.0,
    // )?;
    // let result = threshrlded.clone();

    // let mut dilated = Mat::default();
    // let kernel = opencv::imgproc::get_structuring_element(
    //     opencv::imgproc::MORPH_RECT,
    //     opencv::core::Size::new(3, 3),
    //     opencv::core::Point::new(-1, -1),
    // )?;
    // opencv::imgproc::dilate(
    //     &result,
    //     &mut dilated,
    //     &kernel,
    //     opencv::core::Point::new(-1, -1),
    //     1,
    //     opencv::core::BORDER_CONSTANT,
    //     opencv::imgproc::morphology_default_border_value()?,
    // )?;
    // let result = dilated.clone();

    opencv::imgcodecs::imwrite(output_path, &result, &opencv::core::Vector::<i32>::new())?;

    Ok(())
}

fn find_card_locations_opencv_template(
    image_path: &str,
    template_path: &str,
    threshold: f32,
) -> Result<Vec<ScreenRect>, Box<dyn Error>> {
    let src = opencv::imgcodecs::imread(image_path, opencv::imgcodecs::IMREAD_COLOR).map_err(|err| Box::new(err) as Box<dyn Error>)?;
    let mut gray_src = Mat::default();
    opencv::imgproc::cvt_color(&src, &mut gray_src, opencv::imgproc::COLOR_BGR2GRAY, 0)?;

    let template = opencv::imgcodecs::imread(template_path, opencv::imgcodecs::IMREAD_COLOR).map_err(|err| Box::new(err) as Box<dyn Error>)?;
    let mut gray_template = Mat::default();
    opencv::imgproc::cvt_color(&template, &mut gray_template, opencv::imgproc::COLOR_BGR2GRAY, 0)?;

    let mut result = Mat::default();
    opencv::imgproc::match_template(
        &gray_src,
        &gray_template,
        &mut result,
        opencv::imgproc::TM_CCORR_NORMED,
        &Mat::default(),
    )?;

    let mut normalized_result = Mat::default();
    opencv::core::normalize(
        &result,
        &mut normalized_result,
        0.0,
        1.0,
        opencv::core::NORM_MINMAX,
        -1,
        &Mat::default(),
    )?;

    let mut card_locations = Vec::new();
    let template_width = template.size()?.width;
    let template_height = template.size()?.height;

    for y in 0..normalized_result.size()?.height {
        for x in 0..normalized_result.size()?.width {
            let score = *normalized_result.at_2d::<f32>(y, x)?;
            if score > threshold {
                // dbg!(score);
                let x = x as i32;
                let y = y as i32;
                let rect = ScreenRect::new(x, y, template_width, template_height);
                card_locations.push(rect);
            }
        }
    }

    Ok(card_locations)
}

fn find_card_locations_opencv_edge(image_path: &str) -> Result<Vec<ScreenRect>, Box<dyn Error>> {
    let src = opencv::imgcodecs::imread(
        image_path,
        opencv::imgcodecs::IMREAD_COLOR)
        .map_err(|err| Box::new(err) as Box<dyn Error>)?;
    let mut gray_src = Mat::default();
    opencv::imgproc::cvt_color(
        &src,
        &mut gray_src,
        opencv::imgproc::COLOR_BGR2GRAY,
        0
    )?;

    let mut blurred = Mat::default();
    opencv::imgproc::gaussian_blur(
        &gray_src,
        &mut blurred,
        opencv::core::Size::new(3, 3),
        0.0,
        0.0,
        opencv::core::BORDER_DEFAULT,
    )?;

    let mut edges = Mat::default();
    opencv::imgproc::canny(
        &blurred,
        &mut edges,
        75.0,
        200.0,
        3,
        false,
    )?;

    let mut dilated_edges = Mat::default();
    let kernel = opencv::imgproc::get_structuring_element(
        opencv::imgproc::MORPH_RECT,
        opencv::core::Size::new(3, 3),
        opencv::core::Point::new(-1, -1),
    )?;
    opencv::imgproc::dilate(
        &edges,
        &mut dilated_edges,
        &kernel,
        opencv::core::Point::new(-1, -1),
        1,
        opencv::core::BORDER_CONSTANT,
        opencv::imgproc::morphology_default_border_value()?,
    )?;

    let mut contours = opencv::types::VectorOfVectorOfPoint::new();
    opencv::imgproc::find_contours(
        &dilated_edges,
        &mut contours,
        opencv::imgproc::RETR_LIST,
        opencv::imgproc::CHAIN_APPROX_SIMPLE,
        opencv::core::Point::new(0, 0),
    )?;

    let mut rectangles = Vec::new();

    dbg!(contours.len());

    for i in 0..contours.len() {
        let contour = contours.get(i).unwrap();
        let mut approx = opencv::types::VectorOfPoint::new();
        opencv::imgproc::approx_poly_dp(
            &contour,
            &mut approx,
            0.01 * opencv::imgproc::arc_length(&contour, true)?,
            true)?;

        if approx.len() == 4 && opencv::imgproc::contour_area(&approx, false)? > 1000.0 {
            let points: Vec<opencv::core::Point> = approx.to_vec();
            let rect = opencv::core::Rect::from_points(points[0], points[2]);

            let screen_rect = ScreenRect::new(rect.x, rect.y, rect.width, rect.height);
            rectangles.push(screen_rect);
        }
    }

    Ok(rectangles)
}

fn find_card_locations_imageproc(
    image_path: &str,
    template_path: &str,
    threshold: f32,
) -> Result<Vec<ScreenRect>, Box<dyn Error>> {
    let image = image::open(image_path).map_err(|err| Box::new(err) as Box<dyn Error>)?.to_luma8();
    let template = image::open(template_path).map_err(|err| err.to_string())?.to_luma8();

    let method = imageproc::template_matching::MatchTemplateMethod::SumOfSquaredErrorsNormalized;
    let score_map = imageproc::template_matching::match_template(&image, &template, method);

    let mut card_locations = Vec::new();
    let template_width = template.width();
    let template_height = template.height();

    for y in 0..score_map.height() {
        for x in 0..score_map.width() {
            let score = score_map.get_pixel(x, y)[0];
            if score > threshold {
                let x = x as i32;
                let y = y as i32;
                let rect = imageproc::rect::Rect::at(x, y).of_size(template_width, template_height);
                card_locations.push(rect);
            }
        }
    }

    Ok(card_locations.iter().map(|&rect| ScreenRect {
        x: rect.left() as i32,
        y: rect.top() as i32,
        width: rect.width() as i32,
        height: rect.height() as i32, 
    }).collect::<Vec<_>>())
}
