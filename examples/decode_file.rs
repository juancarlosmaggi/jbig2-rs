//! Example: Decode a JBIG2 file to raw bitmap data.
use jbig2_rs::Jbig2Document;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.jb2> [output.bin]", args[0]);
        eprintln!("\nDecodes a JBIG2 file and saves the first page as raw bitmap data.");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 3 {
        &args[2]
    } else {
        "output.bin"
    };

    println!("Reading JBIG2 file: {}", input_path);

    let data = fs::read(input_path)?;

    let document = Jbig2Document::parse(&data)?;

    println!("Document parsed successfully!");
    println!("Number of pages: {}", document.page_count());

    if let Some(page) = document.get_page(0) {
        println!("\nPage 0 information:");
        println!(
            "  Dimensions: {}x{} pixels",
            page.page_info.width, page.page_info.height
        );

        let bitmap_data = page.to_image_data();

        println!("  Bitmap size: {} bytes", bitmap_data.len());
        println!("  Bits per pixel: 1 (monochrome)");

        fs::write(output_path, &bitmap_data)?;
        println!("\nSaved bitmap data to: {}", output_path);

        println!("\nThe output file contains raw bitmap data:");
        println!("  - 1 bit per pixel (0=white, 1=black)");
        println!("  - Packed into bytes (8 pixels per byte)");
        println!("Stride: {} bytes", page.page_info.width.div_ceil(8));
        println!("  - Total rows: {}", page.page_info.height);
    } else {
        eprintln!("Error: No pages found in document");
        std::process::exit(1);
    }

    Ok(())
}
