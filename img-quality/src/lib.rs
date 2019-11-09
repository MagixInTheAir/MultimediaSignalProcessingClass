use image::GenericImageView;

// Get value from flat 2D array, or 0 if out of bounds
fn get_val<T>(arr: &Vec<T>, width: usize, x: usize, y: usize) -> T where T : Default + Copy {
    if x + (y*width) >= arr.len() { return T::default(); }
    return arr[x + (y*width)];
}

// Human vision filter
pub fn get_hvf() -> (Vec<f64>, usize) {
    let std_dev : f64 = 1.3;   // Human vision model
    let hvf_size : usize = 9;    // Arbitrary
    let length = hvf_size*hvf_size;
    let mut filter : Vec<f64> = vec![];
    for i in 0..length {
        // Position in the array
        let x : usize = i % hvf_size;
        let y : usize = (i-x) / hvf_size;
        // println!("pos : ({:?}, {:?})", x, y);

        // Position in the array centered around zero
        let x_off : i32 = x as i32 - ((hvf_size as f64/2.0) + 1.0).floor() as i32;
        let y_off : i32 = y as i32 -((hvf_size as f64/2.0) + 1.0).floor() as i32;
        // println!("pos_off : ({:?}, {:?})", x_off, y_off);

        // Gauss function
        let distance_sq : usize = x_off.pow(2) as usize + y_off.pow(2) as usize;
        let exp_part : f64 = -(distance_sq as f64)/(2.0*std_dev*std_dev);
        let fact_part : f64 = 1.0/(2.0*std_dev);
        // println!("vals : {:?}, {:?}, {:?}, {:?} ", distance_sq, exp_part, fact_part, fact_part*exp_part.exp());

        // Save
        filter.push(fact_part * exp_part.exp());
    }

    return (filter, hvf_size);
}

// Compute human-filtered peak-to-peak signal-to-noise ratio
pub fn hpsnr(original_img : &image::DynamicImage, new_img : &image::DynamicImage) -> Result<f64, String> {
    let (filter, hvf_size) = get_hvf();
    
    // Dimensions check
    if original_img.dimensions() != new_img.dimensions() { return Err(format!("Size doesn't match : {:?} vs {:?}", original_img.dimensions(), new_img.dimensions())); }
    let img_width = original_img.dimensions().0 as usize;
    let img_height = original_img.dimensions().1 as usize;
    let original = original_img.raw_pixels();
    let new = new_img.raw_pixels();
    
    // Double loop for the denominator
    let mut doublesum = 0.0;
    for i in 0..img_width {
        for j in 0..img_height {

            let mut sum : f64 = 0.0;
            for m in 0..hvf_size {
                for n in 0..hvf_size {
                    sum += get_val(&filter, hvf_size, m, n) * (get_val(&original, img_width, i+m, j+n) as f64 - get_val(&new, img_width, i+m, j+n) as f64);
                }
            }
            doublesum += sum.powf(2.0);

        }
    }
    
    // Final calculation
    let hpsnr = 10.0 * ((img_width as f64 * img_height as f64 * 255.0 * 255.0) / doublesum).log(10.0);
    return Ok(hpsnr);
}

// Compute mean square error
pub fn mse(img1 : &image::DynamicImage, img2 : &image::DynamicImage) -> Result<f32, String> {
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
        sum += (pix1 - pix2).powf(2.0);
    }

    return Ok(sum / (pix1.len() as f32));
}
