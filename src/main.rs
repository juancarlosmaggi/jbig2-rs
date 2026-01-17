use clap::Parser;
use jbig2_rs::image::Jbig2Document;
use std::fs;
use std::path::Path;

/// Command-line entry point for decoding a JBIG2 file into per-page images.
#[derive(Parser)]
#[command(name = "jbig2-decoder")]
#[command(about = "Decode JBIG2 files to images")]
#[command(version)]
struct Args {
    /// Input JBIG2 file path
    #[arg(short, long)]
    input: String,

    /// Output directory (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: String,

    /// Output file prefix (defaults to input filename without extension)
    #[arg(short = 'p', long)]
    prefix: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        eprintln!("Error: Input file '{}' does not exist", args.input);
        std::process::exit(1);
    }

    // Validate input path early to give a clear CLI error before decoding.
    let data = fs::read(&args.input)?;

    let document = Jbig2Document::parse(&data)?;

    if document.page_count() == 0 {
        eprintln!("Warning: No pages found in the JBIG2 document");
        return Ok(());
    }

    // Ensure the output directory is available for image writes.
    let output_dir = Path::new(&args.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Default the prefix to the input file stem to keep output names stable.
    let prefix = args.prefix.unwrap_or_else(|| {
        input_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    println!(
        "Decoding {} pages from '{}'",
        document.page_count(),
        args.input
    );

    for (page_index, page) in document.pages.iter().enumerate() {
        let image_data = page.to_image_data();
        let width = page.page_info.width;
        let height = page.page_info.height;

        // Convert the raw 1bpp page buffer into a grayscale image for PNG output.
        let img = image::GrayImage::from_raw(width, height, image_data)
            .ok_or("Failed to create image from pixel data")?;

        let output_filename = if document.page_count() == 1 {
            format!("{}.png", prefix)
        } else {
            format!("{}_page_{:03}.png", prefix, page_index + 1)
        };

        let output_path = output_dir.join(output_filename);

        img.save(&output_path)?;
        println!(
            "Saved page {} to '{}'",
            page_index + 1,
            output_path.display()
        );
    }

    println!("Successfully decoded {} pages", document.page_count());
    Ok(())
}
