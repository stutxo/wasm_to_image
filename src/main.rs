use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::{ImageBuffer, ImageFormat, Luma};
use oxipng::{optimize_from_memory, Options};
use std::env;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: program <path_to_wasm_file>".into());
    }
    let wasm_file_path = PathBuf::from(&args[1]);

    let mut file = File::open(&wasm_file_path)?;
    let mut wasm_bytes = Vec::new();
    file.read_to_end(&mut wasm_bytes)?;

    let length = wasm_bytes.len() as u32;
    let length_bytes = length.to_le_bytes();
    let mut wasm_with_length = Vec::from(length_bytes);
    wasm_with_length.extend(wasm_bytes);

    let size = wasm_with_length.len();
    let width = (size as f64).sqrt().ceil() as usize;
    let height = (size as f64 / width as f64).ceil() as usize;
    let zero_padding = width * height - size;

    wasm_with_length.extend(vec![0; zero_padding]);

    let img =
        ImageBuffer::<Luma<u8>, Vec<u8>>::from_vec(width as u32, height as u32, wasm_with_length)
            .ok_or("Failed to create image buffer")?;

    // Convert the image to PNG format and capture in a byte vector
    let mut png_bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;

    let options = Options::from_preset(7);
    let png_bytes = optimize_from_memory(&png_bytes, &options)?;

    // Encode the bytes into a base64 string using the standard base64 engine
    let base64_string = STANDARD.encode(&png_bytes);

    // Create a data URI
    let data_uri = format!("data:image/png;base64,{}", base64_string);

    // Output the data URI
    println!("{}", data_uri);

    let output_filename = wasm_file_path.with_extension("png");
    let mut output_file = File::create(output_filename)?;
    output_file.write_all(&png_bytes)?;

    Ok(())
}
