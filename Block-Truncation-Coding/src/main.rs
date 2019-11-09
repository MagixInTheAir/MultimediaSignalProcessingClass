use structopt::StructOpt;
use image::GenericImageView;
use num_traits::pow::Pow;
use std::string::String;
use img_quality;
use std::fmt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}

#[derive(Clone)]
struct VecBlock {
    width : usize, height : usize,
    pixels : Vec<u8>
}

#[derive(Debug, Clone)]
struct BtcBlock {
    high_color: u8, low_color: u8,
    width : usize, height : usize,
    pixels: Vec<u8>
}

impl fmt::Display for BtcBlock { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return write!(f, "VecBlock ({}, {}) with a = {}, b = {} has pixels {:?}", self.width, self.height, self.high_color, self.low_color, self.pixels);
}}

struct BtcImage {
    block_count_x: usize,
    width : usize, height : usize,
    blocks: Vec<BtcBlock>
}

fn no_overflow_cast(x : f64) -> u8 {
    if x < 0.0 { 0 } else if x > 255.0 { 255 } else { x as u8 }
}

fn get_block_nb(i : usize, img_width : usize, block_width : usize, block_height : usize, block_count_x : usize) -> (usize, usize, usize){
    let pos_x = (i % img_width) as usize;
    let pos_y = ((i - pos_x) / img_width) as usize;

    let block_x = pos_x / block_width;
    let block_y = pos_y / block_height;
    let block_count = block_x + (block_y * block_count_x);
    //println!("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, ", block_x, block_y, block_count_x, block_count_y, pos_x, pos_y);

     let block_pos_x = pos_x % block_width;
    let block_pos_y = pos_y % block_height;

    return (block_count, block_pos_x, block_pos_y);
}

fn btc_encode(image: image::DynamicImage, block_width: usize, block_height: usize) -> BtcImage {
    let pixels : Vec<u8> = image.raw_pixels();
    let size = image.dimensions();
    let img_width = size.0 as usize;
    let img_height = size.1 as usize;

    // Build the blocks
    let block_count_x = (img_width as f64 / block_width as f64).ceil() as usize;
    let block_count_y = (img_height as f64 / block_height as f64).ceil() as usize;
    let block_count_total = block_count_x * block_count_y;
    let mut blocks: Vec<VecBlock> = vec![VecBlock{height : 0, width : 0, pixels : vec![]}; block_count_total];

    for i in 0..pixels.len() {
        
        let (block_pos, block_pos_x, block_pos_y) = get_block_nb(i, img_width, block_width, block_height, block_count_x);
        let block = &mut blocks[block_pos];

        if block_pos_x + 1 > block.width { block.width = block_pos_x + 1; }
        if block_pos_y + 1 > block.height { block.height = block_pos_y + 1; }
        block.pixels.push(pixels[i]);
    }

    // println!("{:?}, {:?}", blocks[0].len(), blocks[0]);
    // println!("{:?}", pixels.len());

    let mut result : BtcImage = BtcImage{ block_count_x: block_count_x, blocks: vec![], width: img_width, height: img_height};
    for i in 0..blocks.len() {
        let block = &blocks[i];

        let h : f64 = block.pixels.iter().map(|&x| x as f64).sum::<f64>() as f64 / block.pixels.len() as f64;
        let h2 : f64  = block.pixels.iter().map(|&val| (val as f64).pow(2)).sum::<f64>() / block.pixels.len() as f64;
        let theta : f64 = (h2 - h.pow(2) as f64).sqrt();
        let q = block.pixels.iter().filter(|&&x| x as f64 >= h).count() as f64;
        let m = block.pixels.len() as f64;

        let a = if m-q == 0.0 { h } else { h - (theta * (q/(m-q)).sqrt()) };
        let b = if q == 0.0 { h } else { h + (theta * ((m-q)/q).sqrt()) };
        // println!("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, ", i, h, h2, theta, q, m, a, b);

        //let pix : BitVec = block.pixels.iter().map(|&x| if x < h as u8 { 0 } else { 1 }).collect::<Vec<u8>>().into();
        let pix : Vec<u8> = block.pixels.iter().map(|&x| if x < h as u8 { 0 } else { 1 }).collect::<Vec<u8>>().into();
        let newblock = BtcBlock{pixels: pix, low_color: no_overflow_cast(a), high_color: no_overflow_cast(b), width: block.width, height : block.height};
        //println!("newblock : {}", newblock);
        result.blocks.push(newblock);
        
    }

    return result;
}

fn btc_decode(image: BtcImage) -> image::DynamicImage {
    let mut pixels : Vec<u8> = vec![];   

    let basewidth = image.blocks[0].width;
    let baseheight = image.blocks[0].height;
    let fullsize = image.width * image.height;
    // println!("{:?}, {:?}", basewidth, baseheight);

    for i in 0..fullsize {    

            let (block_pos, block_pos_x, block_pos_y) = get_block_nb(i, image.width, basewidth, baseheight, image.block_count_x);
            let block = &image.blocks[block_pos];
            let pixel_pos = (block_pos_y * block.width) + block_pos_x;

            let pixel = block.pixels[pixel_pos];
            pixels.push(if pixel == 1 { block.high_color } else { block.low_color });


    }

    // println!("{:?}", pixels.len());

    return image::DynamicImage::ImageLuma8(image::ImageBuffer::from_vec(image.width as u32, image.height as u32, pixels).unwrap());
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

    let output_4 = btc_decode(btc_encode(img.clone(), 4,4));
    let output_8 = btc_decode(btc_encode(img.clone(), 8,8));
    let output_16 = btc_decode(btc_encode(img.clone(), 16,16));
    let output_32 = btc_decode(btc_encode(img.clone(), 32,32));

    println!("HPSNR 4x4 : {}", img_quality::hpsnr(&img, &output_4).unwrap());
    println!("HPSNR 8x8 : {}", img_quality::hpsnr(&img, &output_8).unwrap());
    println!("HPSNR 16x16 : {}", img_quality::hpsnr(&img, &output_16).unwrap());
    println!("HPSNR 32x32 : {}", img_quality::hpsnr(&img, &output_32).unwrap());



    println!("Saving results");
    output_4.save("./output4.png").unwrap();
    output_8.save("./output8.png").unwrap();
    output_16.save("./output16.png").unwrap();
    output_32.save("./output32.png").unwrap();

    
}
