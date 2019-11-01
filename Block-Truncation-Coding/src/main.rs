use structopt::StructOpt;
use image::GenericImageView;
use num_traits::pow::Pow;
use bitvec::prelude::*;
use std::convert::From;
use std::string::String;

mod hpsnr;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}


struct BtcBlock {
    highCcolor: u8,
    lowColor: u8,
    pixels: BitVec
}

struct BtcImage {
    imgSize: (u32, u32),
    blockSize: (u32, u32),
    blocks: Vec<BtcBlock>
}


fn btc_encode(image: image::DynamicImage, blockWidth: usize, blockHeight: usize) -> BtcImage {
    let mut pixels : Vec<u8> = image.raw_pixels();
    let size = image.dimensions();
    let img_width = size.0 as usize;
    let img_height = size.1 as usize;

    


    // Build the blocks
    let block_count_x = (img_width as f64 / blockWidth as f64).ceil() as usize;
    let block_count_y = (img_height as f64 / blockHeight as f64).ceil() as usize;
    let block_count_total = block_count_x*block_count_y;
    let mut blocks: Vec<Vec<u8>> = vec![vec![]; block_count_total];
    for i in 0..pixels.len() {
        let pos_x = (i % img_width) as usize;
        let pos_y = ((i - pos_x) / img_width) as usize;

        let block_x = pos_x / blockWidth;
        let block_y = pos_y / blockHeight;
        let block_count = block_x + (block_y * block_count_y);

        //println!("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, ", block_x, block_y, block_count_x, block_count_y, pos_x, pos_y);

        blocks[block_count].push(pixels[i]);
    }

    //println!("{:?}", blocks[0].len());


    for i in 0..blocks.len() {
        
        let block = blocks[i];

        let avg = block.iter().sum() as f64 / block.len() as f64;
        let sd  = (block.iter().map(|val| (*val as f64 - avg).pow(2)).sum() as f64 / block.len() as f64).sqrt();
        


    }

    return BtcImage{ imgSize: (0,0), blockSize: (0,0), blocks: vec![]};
}

//fn btc_decode(image: BtcImage) -> image::DynamicImage {
    
//}


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

    /*let output = btc_decode(btc_encode(img.clone()));

    println!("MSE : {}", mse(&img, &output).unwrap());



    println!("Saving result");
    output.save("./output.png").unwrap();*/
    btc_encode(img.clone(), 32, 32);

    
}
