use std::error::Error;

use opencv::prelude::*;

use super::screen::ScreenRect;

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
