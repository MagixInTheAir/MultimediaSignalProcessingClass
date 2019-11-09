use structopt::StructOpt;
use image::GenericImageView;
use std::string::String;
use img_quality;
use std::fmt;
mod arrays;




#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}

#[derive(Clone, Debug)]
struct VecBlock {
    width : usize, height : usize,
    pixels : Vec<u8>
}

#[derive(Debug, Clone)]
struct OdbtcBlock {
    high_color: u8, low_color: u8,
    pixels: Vec<u8>,
    width: usize, height : usize
}

impl fmt::Display for OdbtcBlock { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return write!(f, "OdbtcBlock with a = {}, b = {} has pixels {:?}", self.high_color, self.low_color, self.pixels);
}}

struct OdbtcImage {
    block_count_x: usize,
    width : usize, height : usize,
    blocks: Vec<OdbtcBlock>
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

fn find_dither(size: usize) -> (Vec<f64>, f64, f64) {
    // Find dither array
    let dither_arr : Vec<f64> = match size {
        4 => arrays::DITHER_4.to_vec(),
        8 => arrays::DITHER_8.to_vec(),
        16 => arrays::DITHER_16.to_vec(),
        32 => arrays::DITHER_32.to_vec(),
        64 => arrays::DITHER_64.to_vec(),
        _ => panic!("BAD BLOCK SIZE")
    };
    let dither_max = dither_arr.iter().cloned().fold(0./0., f64::max);
    let dither_min = dither_arr.iter().cloned().fold(0./0., f64::min);

    return (dither_arr, dither_max, dither_min);
}

fn gen_dither(dither_arr : Vec<f64>, max: u8, min: u8, dither_max: f64, dither_min: f64) -> Vec<f64> { 
    return dither_arr.into_iter().map(|x| ((max as f64 - min as f64) * (( x - dither_min  )/( dither_max - dither_min ))) + min as f64).collect();
}

fn get_dither(size :usize, max: u8, min: u8) -> Vec<f64> {

        
        let (dither_arr, dither_max, dither_min) = find_dither(size);
        return gen_dither(dither_arr, max, min, dither_max, dither_min);

        

}

fn build_blocks(pixels : &Vec<u8>, img_width: usize, img_height: usize, block_size: usize) -> (Vec<VecBlock>, usize, usize) {
    
    let block_count_x = (img_width as f64 / block_size as f64).ceil() as usize;
    let block_count_y = (img_height as f64 / block_size as f64).ceil() as usize;
    let block_count_total = block_count_x * block_count_y;
    let mut blocks: Vec<VecBlock> = vec![VecBlock{height : 0, width : 0, pixels : vec![]}; block_count_total];

    for i in 0..pixels.len() {
        
        let (block_pos, block_pos_x, block_pos_y) = get_block_nb(i, img_width, block_size, block_size, block_count_x);
        let block = &mut blocks[block_pos];

        if block_pos_x + 1 > block.width { block.width = block_pos_x + 1; }
        if block_pos_y + 1 > block.height { block.height = block_pos_y + 1; }
        block.pixels.push(pixels[i]);
    }

    // println!("Generated {:?} VecBlocks", blocks.len());

    return (blocks, block_count_x, block_count_y);
}

fn encode_blocks(blocks : &Vec<VecBlock>, block_size: usize, block_count_x: usize, img_width: usize, img_height: usize) -> OdbtcImage {
    let mut result : OdbtcImage = OdbtcImage{block_count_x: block_count_x, blocks: vec![], width: img_width, height: img_height};
    for i in 0..blocks.len() {
        let block = &blocks[i];

        // Compute min-max
        let &max = block.pixels.iter().max().unwrap();
        let &min = block.pixels.iter().min().unwrap();

        let dither_arr = get_dither(block_size, max, min);


        // Threshold to min and max
        let mut pix = block.pixels.clone();
        for i in 0..block.pixels.len() {
            pix[i] = if pix[i] as f64 > dither_arr[i] { 1 } else { 0 };
        }

       
        let newblock = OdbtcBlock{pixels: pix, low_color: min, high_color: max, width: block.width, height : block.height};
        // println!("newblock : {}", newblock);
        result.blocks.push(newblock);
        
    }

    // println!("Generated {:?} blocks", result.blocks.len());

    return result;
}

fn odbtc_encode(image: image::DynamicImage, block_size : usize) -> OdbtcImage {
    println!("Encoding...");

    let pixels : Vec<u8> = image.raw_pixels();
    let size = image.dimensions();
    let width = size.0 as usize;
    let height = size.1 as usize;
    println!("{:?} {:?}", width, height);

    let (blocks, block_count_x, _) = build_blocks(&pixels, width, height, block_size);
    return encode_blocks(&blocks, block_size, block_count_x, width, height);
    
}

fn odbtc_decode(image: OdbtcImage) -> image::DynamicImage {
    println!("Decoding...");

    let mut pixels : Vec<u8> = vec![];   

    let basewidth = image.blocks[0].width;
    let baseheight = image.blocks[0].height;
    let fullsize = image.width * image.height;


    // println!("{:?}, {:?}", basewidth, baseheight);
    // println!("{:?} {:?} {:?}", image.width, image.height, fullsize);
    
    for i in 0..fullsize {    

        let (block_pos, block_pos_x, block_pos_y) = get_block_nb(i, image.width, basewidth, baseheight, image.block_count_x);
        let block = &image.blocks[block_pos];

        let pixel_pos = (block_pos_y * block.width) + block_pos_x;
        // println!("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, ", block_pos_x, block_pos_y, block.width, pixel_pos, block.pixels.len(), dither_arr.len());
        pixels.push(if block.pixels[pixel_pos] == 1 { block.high_color } else { block.low_color });


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

    let output_4 = odbtc_decode(odbtc_encode(img.clone(), 4));
    let output_8 = odbtc_decode(odbtc_encode(img.clone(), 8));
    let output_16 = odbtc_decode(odbtc_encode(img.clone(), 16));
    let output_32 = odbtc_decode(odbtc_encode(img.clone(), 32));

    println!("Saving results");
    output_8.save("./output8.png").unwrap();
    output_4.save("./output4.png").unwrap();
    output_16.save("./output16.png").unwrap();
    output_32.save("./output32.png").unwrap();

    println!("HPSNR 4x4 : {}", img_quality::hpsnr(&img, &output_4).unwrap());
    println!("HPSNR 8x8 : {}", img_quality::hpsnr(&img, &output_8).unwrap());
    println!("HPSNR 16x16 : {}", img_quality::hpsnr(&img, &output_16).unwrap());
    println!("HPSNR 32x32 : {}", img_quality::hpsnr(&img, &output_32).unwrap());
    
}
