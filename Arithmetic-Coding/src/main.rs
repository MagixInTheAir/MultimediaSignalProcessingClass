use structopt::StructOpt;
use std::fs;
use std::collections::HashMap;
use rug::{Float, ops::AssignRound, float::Round};
use std::mem;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE")]
    file: String
}


fn find_frequencies(data : &String) -> HashMap<char, u32> {
    let mut frequencies : HashMap<char, u32> = HashMap::new();

    for c in data.chars() {
        // Read previous character count
        let mut cur_count : u32 = 0;
        if let Some(&count) = frequencies.get(&c) { cur_count = count; }

        // Increase it
        let entry = frequencies.entry(c).or_insert(cur_count);
        *entry = *entry + 1;
    }

    return frequencies;
}

fn compression_ratio(before: &String, after: &Float) -> f64 {
    let before_size = mem::size_of_val(&before[..]);
    let after_size = after.prec() + 32;
    println!("before : {:?}, after : {:?}", before_size, after_size);
    return after_size as f64 / before_size as f64;    

}

fn find_bounds(freqs: &HashMap<char, u32>, filesize: u32, step_precision: u32) -> HashMap<char, (Float, Float)> {
    let mut bounds : HashMap<char, (Float, Float)> = HashMap::new();

    let mut prevhigh = Float::with_val(step_precision, 0.0);
    for (&c, &count) in freqs {
        let low = prevhigh;
        let high = Float::with_val(step_precision, low.clone() + (count as f64 / filesize as f64));
        prevhigh = Float::with_val(step_precision, high.clone());

        bounds.insert(c, (low, high));
    }

    return bounds;
}

fn arithmetic_encode(file: &String, bounds: &HashMap<char, (Float, Float)>, step_precision: u32) -> Float {

    // Start with 1-bit precision
    let mut high : Float = Float::with_val(32, 1);
    let mut low : Float  = Float::with_val(32, 0);

    for c in file.chars() {

        // Increase the precision of the floats
        high.set_prec(high.prec() + step_precision);
        low.set_prec(low.prec() + step_precision);

        //println!("{:?} : {:?}, {:?}, {:?}, {:?}, ", c, low.prec(), high.prec(), high, low);
        
        let range = high.clone() - low.clone();

        //println!("{:?} : {:?}, {:?}, {:?}, {:?}, {:?}, ", c, low.prec(), high.prec(), range, high, low);

        high.assign_round(low.clone() + (range.clone() * bounds.get(&c).unwrap().1.clone()), Round::Down);
        low.assign_round(low.clone() + (range.clone() * bounds.get(&c).unwrap().0.clone()), Round::Up);
        //println!("{:?} : {:?}, {:?}, {:?}, {:?}, {:?}, ", c, low.prec(), high.prec(), range, high, low); 
    }

    return (high + low)/2;
}

fn arithmetic_decode(encoded : &Float, bounds: &HashMap<char, (Float, Float)>, filesize : u32, step_precision: u32) -> String {
    let mut result : String = String::new();
    let mut data = encoded.clone();

    // Initial precision is 32, so get back to that
    while data.prec() > 32 {
        for(c, bounds) in bounds {
            if bounds.1 > data && bounds.0 < data {
                println!("{:?} : {:?} at prec {:?}, bounds are {:?} - {:?}", c, data, data.prec(), bounds.0.clone(), bounds.1.clone());
                result.push(*c);
                data.assign_round((data.clone() - bounds.0.clone()) / (bounds.1.clone() - bounds.0.clone()), Round::Up);
                data.set_prec(data.prec() - step_precision);
                break;
            }
        }
    }

    return result;
}

fn main() {
    // Parse arguments
    let opt = Opt::from_args();

    println!("Reading file");
    let contents = fs::read_to_string(opt.file).unwrap();
    let filesize = contents.len() as u32;

    // https://stackoverflow.com/questions/7150035/ and add a small pad :)
    let step_precision = (filesize as f64).log2().ceil() as u32;
    println!("Step precision : {:?}", step_precision);

    // Find frequency
    let frequencies = find_frequencies(&contents);
    let bounds = find_bounds(&frequencies, filesize, step_precision);
    println!("{:?}\n{:?}", frequencies.clone(), bounds.clone());

    // Encode file
    let encoded = arithmetic_encode(&contents, &bounds, step_precision);
    println!("encoder result : {:?}", encoded);

    // Compute compression ratio
    let ratio = compression_ratio(&contents, &encoded);
    println!("Compression ratio : {:?}", ratio);

    // Decode file
    let decoded = arithmetic_decode(&encoded, &bounds, filesize, step_precision);
    println!("decoder result : {:?}", decoded);

    println!("Saving result");
    fs::write("output.txt", &decoded[..]).unwrap();

    
}
