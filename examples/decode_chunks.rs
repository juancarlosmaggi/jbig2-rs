//! Example: Decode JBIG2 from chunked data buffers.
use jbig2_rs::{Jbig2Chunk, Jbig2Document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("JBIG2 Chunk-Based Decoding Example\n");

    // Simulated example: in real usage, these would come from an external source.

    // Example chunk 1: Global segment data (shared across pages).
    let global_data = vec![
        // Empty data for demonstration.
    ];

    // Example chunk 2: Page-specific segment data.
    let page_data = vec![
        // Empty data for demonstration.
    ];

    // Create chunk descriptors.
    let chunks = vec![
        Jbig2Chunk {
            data: global_data.clone(),
            start: 0,
            end: global_data.len(),
        },
        Jbig2Chunk {
            data: page_data.clone(),
            start: 0,
            end: page_data.len(),
        },
    ];

    println!("Processing {} chunks...", chunks.len());

    match Jbig2Document::parse_chunks(&chunks) {
        Ok(document) => {
            println!("Document parsed successfully!");
            println!("Number of pages: {}", document.page_count());

            for i in 0..document.page_count() {
                if let Some(page) = document.get_page(i) {
                    println!("\nPage {}:", i);
                    println!(
                        "  Dimensions: {}x{} pixels",
                        page.page_info.width, page.page_info.height
                    );

                    let bitmap_data = page.to_image_data();
                    println!("  Bitmap size: {} bytes", bitmap_data.len());
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing chunks: {}", e);
            println!("\nNote: This example uses empty data for demonstration.");
            println!("In a real application, you would:");
            println!("  1. Extract JBIG2 streams from the source container");
            println!("  2. Identify global vs. page-specific segments");
            println!("  3. Create chunks with proper data and offsets");
            println!("  4. Parse chunks to decode the image");
        }
    }

    println!("\n--- Chunk Creation Pattern ---");
    println!("When extracting from containers:");
    println!("  1. Global segment chunk (shared data):");
    println!("     - Symbol dictionaries");
    println!("     - Huffman tables");
    println!("     - Pattern dictionaries");
    println!("\n  2. Page segment chunks (per-page data):");
    println!("     - Page information");
    println!("     - Text regions");
    println!("     - Generic regions");
    println!("     - End markers");

    Ok(())
}
