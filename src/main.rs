use image::{imageops, DynamicImage, GenericImageView, Pixel};
use std::env;
use std::fs;

static USAGE: &'static str = "image2txt WIDTH file ...";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return;
    }
    let char_width: usize = args[1].parse().expect(USAGE);
    for image_path in args.iter().skip(2) {
        let img = match image::open(image_path) {
            Ok(img) => img,
            Err(e) => {
                println!(r#"Can't open "{}" as image. {}"#, image_path, e);
                continue;
            }
        };
        let txt = image2string(img, char_width);
        let output_path = [image_path, ".txt"].concat();
        fs::write(output_path, txt).expect("cant't write file");
    }
}

fn image2string(img: DynamicImage, char_width: usize) -> String {
    let img = img.into_luma();
    let (width_origin, height_origin) = img.dimensions();
    let char_width: usize = char_width;
    let char_height: usize =
        (height_origin as f64 * (char_width as f64 * 2.0 / width_origin as f64)) as usize / 4;
    let width = char_width * 2;
    let height = char_height * 4;

    let img = imageops::resize(
        &img,
        width as u32,
        height as u32,
        imageops::FilterType::CatmullRom,
    );

    // compute mean
    // let mut sum = 0f64;
    // for v in img.iter() {
    //     sum += *v as f64;
    // }
    // let mean: u8 = (sum / (width * height) as f64) as u8;

    let threshold = 128;

    let mut string_img = vec![vec![String::new(); char_width]; char_height];

    // 8 pixels to 1 charactar
    for y in 0..char_height {
        for x in 0..char_width {
            let img_patch = img.view(2 * x as u32, 4 * y as u32, 2, 4);
            let mut flag = 0u8;
            for inner_y in 0..4 {
                for inner_x in 0..2 {
                    let bit = img_patch.get_pixel(inner_x, inner_y).channels()[0] > threshold;
                    let cur = if inner_y == 3 {
                        inner_x + inner_y * 2
                    } else {
                        inner_x * 3 + inner_y
                    };
                    let bit = if bit { 1u8 } else { 0u8 };
                    flag |= bit << cur;
                }
            }
            string_img[y as usize][x as usize] = std::char::from_u32(0x2800 + flag as u32)
                .unwrap()
                .to_string();
        }
    }
    /* note: braille bit order (http://www.unicode.org/charts/PDF/U2800.pdf)
     *
     * |0|3|
     * |1|4|
     * |2|5|
     * |6|7|
     *
     */

    string_img
        .iter()
        .map(|r| r.concat())
        .collect::<Vec<String>>()
        .join("\n")
}
