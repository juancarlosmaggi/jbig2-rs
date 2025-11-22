use jbig2_rs::image::Jbig2Document;
use image::{GrayImage, Luma};
use std::fs;


/// Convert JBIG2 bitmap to PNG image
fn save_as_png(bitmap: &jbig2_rs::bitmap::Bitmap, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let width = bitmap.width as u32;
    let height = bitmap.height as u32;
    
    // Create a grayscale image
    let mut img = GrayImage::new(width, height);
    
    // Convert 1-bit bitmap to 8-bit grayscale
    // 0 = white (255), 1 = black (0)
    for y in 0..height {
        for x in 0..width {
            let pixel_value = bitmap.get_pixel(x as usize, y as usize);
            let gray_value = if pixel_value == 0 { 255 } else { 0 };
            img.put_pixel(x, y, Luma([gray_value]));
        }
    }
    
    img.save(output_path)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing 4 JBIG2 test resource files...\n");
    println!("{}", "=".repeat(60));
    
    // Create output directory
    let output_dir = "output";
    fs::create_dir_all(output_dir)?;
    println!("Output directory: {}/\n", output_dir);
    
    let test_files = [
        ("tests/resources/minimal_valid.jb2", "minimal_valid"),
        ("tests/resources/halftone_region.jb2", "halftone_region"),
        ("tests/resources/symbol_dictionary.jb2", "symbol_dictionary"),
        ("tests/resources/text_region.jb2", "text_region"),
    ];
    
    for (index, (filename, name)) in test_files.iter().enumerate() {
        println!("📄 Processing file {} of {}: {}", 
            index + 1, test_files.len(), filename);
        println!("{}", "-".repeat(60));
        
        // Read the file
        let data = match fs::read(filename) {
            Ok(d) => d,
            Err(e) => {
                println!("❌ Error reading file: {}", e);
                continue;
            }
        };
        
        println!("   File size: {} bytes", data.len());
        
        // Parse the JBIG2 document
        match Jbig2Document::parse(&data) {
            Ok(doc) => {
                println!("   ✅ Successfully parsed!");
                println!("   Pages: {}", doc.page_count());
                
                // Process and save each page
                for page_num in 0..doc.page_count() {
                    match doc.get_page(page_num) {
                        Some(page) => {
                            println!("   📄 Page {}: {}x{} pixels", 
                                page_num, page.page_info.width, page.page_info.height);
                            
                            // Save as PNG
                            let output_filename = if doc.page_count() > 1 {
                                format!("{}/{}_{}.png", output_dir, name, page_num)
                            } else {
                                format!("{}/{}.png", output_dir, name)
                            };
                            
                            match save_as_png(&page.bitmap, &output_filename) {
                                Ok(_) => {
                                    println!("      ✅ Saved: {}", output_filename);
                                }
                                Err(e) => {
                                    println!("      ❌ Failed to save PNG: {}", e);
                                }
                            }
                        }
                        None => {
                            println!("   ❌ Page {}: Could not retrieve page data", page_num);
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Parse error: {}", e);
            }
        }
        println!();
    }
    
    println!("{}", "=".repeat(60));
    println!("✅ Processing complete!");
    println!("   Check the '{}' directory for generated PNG images.\n", output_dir);
    
    Ok(())
}
