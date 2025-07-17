// Module for extracting and displaying version information from Cargo.toml
// This module provides functionality to parse the project's Cargo.toml file
// and display key project information (name, version, edition) to the user

use crate::colour_print;
use serde::Deserialize;
use std::fs;
use toml::de::from_str;

// Structure representing the package section of Cargo.toml
#[derive(Debug, Deserialize)]
struct Package {
    name: String,     // Project name
    version: String,  // Project version
    edition: String,  // Rust edition (e.g., "2021")
}

// Structure representing the complete Cargo.toml file
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

/// Parse and display information from the Cargo.toml file
/// Parameters:
///   - file_path: Path to the Cargo.toml file
fn parse_cargo_toml(file_path: &str) {
    // Check if the Cargo.toml file exists
    if !std::path::Path::new(file_path).exists() {
        eprintln!("\t Cargo.toml file not found;\n\t Cannot display version information.\n");
        return;
    }

    // Read the entire content of the Cargo.toml file
    let content = fs::read_to_string(file_path).expect("Failed to read Cargo.toml file");

    // Parse the TOML content into the CargoToml struct
    let cargo_toml: CargoToml = from_str(&content).expect("Failed to parse Cargo.toml");

    // Display the extracted package information using colored output
    let out_message_0 = format!("\t Package name: '{}'.", cargo_toml.package.name);
    colour_print(&out_message_0, "purple");

    let out_message_1 = format!("\t Package version: '{}'.", cargo_toml.package.version);
    colour_print(&out_message_1, "purple");

    let out_message_2 = format!("\t Package edition: '{}'.\n", cargo_toml.package.edition);
    colour_print(&out_message_2, "purple");
}

/// Main entry point for the toml_extract module
/// This function is called from main.rs to display project information
pub fn main() {
    let file_path = "Cargo.toml"; // Path to your Cargo.toml file
    parse_cargo_toml(file_path);
}
