use structopt::StructOpt;
use image::GenericImageView;
use num_traits::pow::Pow;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}


static floydSteinberg : [f32; 4] = [7.0/16.0, 3.0/16.0, 5.0/16.0, 1.0/16.0];
static jarvisAndAl : [f32; 12] = [7.0/48.0, 5.0/48.0, 3.0/48.0, 5.0/48.0, 7.0/48.0, 5.0/48.0, 3.0/48.0, 1.0/48.0, 3.0/48.0, 5.0/48.0, 3.0/48.0, 1.0/48.0];
static stucki : [f32; 12] = [8.0/42.0, 4.0/42.0, 2.0/42.0, 4.0/42.0, 8.0/42.0, 4.0/42.0, 2.0/42.0, 1.0/42.0, 2.0/42.0, 4.0/42.0, 2.0/42.0, 1.0/42.0];




fn hpsnr(img1 : &image::DynamicImage, img2 : &image::DynamicImage) -> f32 {
    return 0.0;
}

fn mse(img1 : &image::DynamicImage, img2 : &image::DynamicImage) -> Result<f32, String> {
    let pix1 = img1.raw_pixels();
    let pix2 = img2.raw_pixels();

    if pix1.len() != pix2.len() {
        return Err("Size doesn't match".to_string());
    }

    let mut sum : f32 = 0.0;
    for pixels in pix1.iter().zip(pix2.iter()) {
        let (p1, p2) = pixels;
        let pix1 = (*p1 as f32) / 255.0;
        let pix2 = (*p2 as f32) / 255.0;
        sum += (pix1 - pix2).pow(2);
    }

    return Ok(sum / (pix1.len() as f32));
}

fn apply_errordiffusion(image: image::DynamicImage, err_ker: &[f32], err_ker_width: usize) -> image::DynamicImage {

    let mut pixels : Vec<f32> = image.raw_pixels().into_iter().map(|pix| { return (pix as f32)/255.0; } ).collect();
    let size = image.dimensions();
    let img_width = size.0 as usize;
    let img_height = size.1 as usize;

    for i in 0..pixels.len() {
            let img_x = i % img_width;
            let img_y = (i - img_x) / img_width;
            let threshold_val = 0.5;

                // 1 : Thresholding
            let result_pixel : f32 = match pixels[i] {
                _ if pixels[i] < threshold_val => 0.0,
                _ if pixels[i] > threshold_val => 1.0,
                _ => 1.0                // ARBITRARY CHOICE !
            };

                // 2 : Error diffusion
            for j in 0..err_ker.len() {

                // Origin of the diffusion weights is the top-center value
                let err_pos = (j as i32) + ((err_ker_width as f32)/2.0).floor() as i32; // Current diffusion weight as if the array was full
                let err_x = err_pos % (err_ker_width as i32);           // Diffusion weight x coordinate
                let err_y = (err_pos - err_x)/(err_ker_width as i32);   // Diffusion weight y coordinate
                let dx = (err_pos % (err_ker_width as i32)) - ((err_ker_width as f32)/2.0).floor() as i32;  // Diffusion weight x difference from origin
                let dy = err_y;                                // Diffusion weight y difference from origin
                // println!("{:?}, {:?}, {:?}, {:?}, {:?}", err_pos, err_x, err_y, dx, dy);

                if
                        img_x as i32 + dx < img_width as i32    // Check horizontal overflow on the right
                    &&  img_x as i32 + dx >= 0            // Check horizontal overflow on the left
                    &&  img_y as i32 + dy < img_height as i32   // Check vertical overflow
                    {
                    
                    // Diffuse the error, upscaled
                    // println!("pixel {:?}/{:?}, size: ({:?}, {:?}), x: ({:?}, {:?}), y: ({:?}, {:?})", i, pixels.len(), img_width, img_height,  img_x, dx, img_y, dy);
                    pixels[((i as i32 + dx) + (img_width as i32 * dy)) as usize]  += (pixels[i] - (result_pixel as f32)) * err_ker[j]; 

                }


            }
            

            pixels[i] = result_pixel;
        };

    let upscaled_pixels = pixels.into_iter().map(|p| (p*255.0).floor() as u8).collect();
    let buffer = image::ImageBuffer::from_vec(img_width as u32, img_height as u32, upscaled_pixels).unwrap();
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

    let output_fs = apply_errordiffusion(img.clone(), &floydSteinberg[..], 3);
    let output_ja = apply_errordiffusion(img.clone(), &jarvisAndAl[..], 5);
    let output_st = apply_errordiffusion(img.clone(), &stucki[..], 5);

    println!("MSE FS : {}", mse(&img, &output_fs).unwrap());
    println!("MSE JA : {}", mse(&img, &output_ja).unwrap());
    println!("MSE ST : {}", mse(&img, &output_st).unwrap());



    println!("Saving result");
    output_fs.save("./output-fs.png").unwrap();
    output_ja.save("./output-ja.png").unwrap();
    output_st.save("./output-st.png").unwrap();

    
}
