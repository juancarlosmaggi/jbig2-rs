//! Example: Decode a JBIG2 file to raw bitmap data.
use clap::Parser;
use jbig2_rs::Jbig2Document;
use std::fs;

/// Decode a JBIG2 file to raw 1bpp bitmap data.
#[derive(Parser)]
struct Args {
    /// Input JBIG2 file path
    input: String,

    /// Output file path (defaults to output.bin)
    output: Option<String>,

    /// Emit decode profiling report to stderr
    #[arg(long)]
    profile: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let input_path = &args.input;
    let output_path = args.output.as_deref().unwrap_or("output.bin");

    println!("Reading JBIG2 file: {}", input_path);

    let data = fs::read(input_path)?;

    let (document, profile) = if args.profile {
        let (document, profile) = Jbig2Document::parse_with_profile(&data)?;
        (document, Some(profile))
    } else {
        (Jbig2Document::parse(&data)?, None)
    };
    if let Some(profile) = profile {
        eprintln!("{}", profile.report());
    }

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
