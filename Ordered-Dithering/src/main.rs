use structopt::StructOpt;
use image::GenericImageView;
use rayon::prelude::*;
use img_quality;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}

static CLASSICAL_4 : [f32; 64] = [
    0.567, 0.635, 0.608, 0.514, 0.424, 0.365, 0.392, 0.486,
    0.847, 0.878, 0.910, 0.698, 0.153, 0.122, 0.090, 0.302,
    0.820, 0.969, 0.941, 0.667, 0.180, 0.031, 0.059, 0.333,
    0.725, 0.788, 0.757, 0.545, 0.275, 0.212, 0.243, 0.455,
    0.424, 0.365, 0.392, 0.486, 0.567, 0.635, 0.608, 0.514,
    0.153, 0.122, 0.090, 0.302, 0.847, 0.878, 0.910, 0.698,
    0.180, 0.031, 0.059, 0.333, 0.820, 0.969, 0.941, 0.667,
    0.275, 0.212, 0.243, 0.455, 0.725, 0.788, 0.757, 0.545 
];

static BAYER_5 : [f32; 64] = [
    0.513, 0.272, 0.724, 0.483, 0.543, 0.302, 0.694, 0.453,
    0.151, 0.755, 0.091, 0.966, 0.181, 0.758, 0.121, 0.936,
    0.634, 0.392, 0.574, 0.332, 0.664, 0.423, 0.604, 0.362,
    0.060, 0.875, 0.211, 0.815, 0.030, 0.906, 0.241, 0.845,
    0.543, 0.302, 0.694, 0.453, 0.513, 0.272, 0.724, 0.483,
    0.181, 0.758, 0.121, 0.936, 0.151, 0.755, 0.091, 0.966,
    0.664, 0.423, 0.604, 0.362, 0.634, 0.392, 0.574, 0.332,
    0.030, 0.906, 0.241, 0.845, 0.060, 0.875, 0.211, 0.815
];

fn apply_dithering(image: image::DynamicImage, dither_array : [f32; 64]) -> image::DynamicImage {

    let mut pixels = image.raw_pixels();
    let size = image.dimensions();
    let img_width = size.0;
    let img_height = size.1;
    let dither_width = 8;
    let dither_height = 8;

    let result = pixels.par_iter_mut().enumerate().map(|(index, pixel)| {
            let img_x = (index as u32) % img_width;
            let img_y = ((index as u32) - img_x) / img_height;

            let dither_x = img_x % dither_width;
            let dither_y = img_y % dither_height;
            let dither_index = dither_x + (dither_y * dither_width);
            let dither_val = dither_array[dither_index as usize];

            let downscaled_pix : f32 = (*pixel as f32)  / 255.0;
            let result_pixel = match downscaled_pix {
                downscaled_pix if downscaled_pix < dither_val => 0,
                downscaled_pix if downscaled_pix > dither_val => 255,
                _ => 255                // ARBITRARY CHOICE !
            };

            return result_pixel;
        }).collect();

    let buffer = image::ImageBuffer::from_vec(img_width, img_height, result).unwrap();
    return image::DynamicImage::ImageLuma8(buffer);
}

fn main() {
    // Parse arguments
    let opt = Opt::from_args();

    println!("Reading image");
    let mut img = image::open(opt.file).unwrap();

    if let image::ColorType::Gray(_) = img.color() {
        println!("Color type ok");
    } else {
        println!("Bad color type, converting");
        img = img.grayscale();
    }

    if let image::DynamicImage::ImageLuma8(_) = img {
        println!("Image type ok");
    } else {
        println!("Wrong image type");
    }

    println!("Saving input image");
    img.save("./input.png").unwrap();

    let classical = apply_dithering(img.clone(), CLASSICAL_4);
    let bayer = apply_dithering(img.clone(), BAYER_5);

    println!("Classical MSE : {}", img_quality::mse(&img, &classical).unwrap());
    println!("Bayer MSE : {}", img_quality::mse(&img, &bayer).unwrap());
    println!("Classical HPSNR : {}", img_quality::hpsnr(&img, &classical).unwrap());
    println!("Bayer HPSNR : {}", img_quality::hpsnr(&img, &bayer).unwrap());


    println!("Saving result");
    classical.save("./output_classical.png").unwrap();
    bayer.save("./output_bayer.png").unwrap();

    
}
