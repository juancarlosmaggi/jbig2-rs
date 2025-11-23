//! Example: Decode JBIG2 from chunks (useful for PDF-embedded JBIG2)
//!
//! This example demonstrates chunk-based decoding, which is useful when JBIG2 data
//! is embedded in PDF files or split across multiple buffers.
use jbig2_rs::{Jbig2Chunk, Jbig2Document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("JBIG2 Chunk-Based Decoding Example\n");

    // Simulated example: In a real PDF extractor, you would read these
    // chunks from the PDF's JBIG2 streams

    // Example chunk 1: Global segment data (shared across pages)
    let global_data = vec![
        // This would contain symbol dictionaries, tables, etc.
        // For demonstration, we'll use empty data
    ];

    // Example chunk 2: Page-specific segment data
    let page_data = vec![
        // This would contain page information and region segments
        // For demonstration, we'll use empty data
    ];

    // Create chunks
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

    // Parse the chunks
    match Jbig2Document::parse_chunks(&chunks) {
        Ok(document) => {
            println!("Document parsed successfully!");
            println!("Number of pages: {}", document.page_count());

            // Process each page
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
            println!("  1. Extract JBIG2 streams from PDF");
            println!("  2. Identify global vs. page-specific segments");
            println!("  3. Create chunks with proper data and offsets");
            println!("  4. Parse chunks to decode the image");
        }
    }

    // Demonstrate proper chunk creation
    println!("\n--- Chunk Creation Pattern ---");
    println!("When extracting from PDFs:");
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
