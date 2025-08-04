//! Debug preprocessor to capture authentic mdBook JSON input
//!
//! This is a simple preprocessor that reads the JSON input that mdBook sends,
//! saves it to a file for analysis, and then passes it through unchanged.
//! This allows us to capture real mdBook preprocessor input format for testing.

use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read all input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Save the input to a file for analysis
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let filename = format!("debug_input_{timestamp}.json");
    std::fs::write(&filename, &input)?;

    eprintln!("Debug preprocessor: Saved input to {filename}");
    eprintln!("Input size: {} bytes", input.len());

    // Try to parse as JSON to verify format
    match serde_json::from_str::<serde_json::Value>(&input) {
        Ok(value) => {
            eprintln!("JSON is valid");
            if let Some(array) = value.as_array() {
                eprintln!("Top-level array with {} elements", array.len());
                if array.len() >= 2 {
                    if let Some(context) = array[0].as_object() {
                        eprintln!("Context keys: {:?}", context.keys().collect::<Vec<_>>());
                    }
                    if let Some(book) = array[1].as_object() {
                        eprintln!("Book keys: {:?}", book.keys().collect::<Vec<_>>());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("JSON parse error: {e}");
        }
    }

    // Pass through the input unchanged to stdout
    io::stdout().write_all(input.as_bytes())?;
    io::stdout().flush()?;

    Ok(())
}
