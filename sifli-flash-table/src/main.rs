use std::{fs::File, io::{Read, Write}, path::PathBuf};

use clap::{Parser, Subcommand};

use sifli_flash_table::ptab;
use sifli_flash_table::ftab;

/// Command line interface for the application
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the PTAB JSON file
    #[arg(short, long, value_name = "FILE")]
    ptab: PathBuf,

    /// Path to the output binary file
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

/// Subcommands for different operations
#[derive(Subcommand)]
enum Commands {
    /// Generate the flash table
    Gen,
}

fn main() {
    let cli = Cli::parse();

    // Debugging output
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // Process the command based on subcommands
    match &cli.command {
        Some(Commands::Gen) => {
            println!("Generating flash table...");

            // Read the PTAB file
            let mut ptab_contents = String::new();
            let mut ptab_file = File::open(&cli.ptab).expect("Failed to open PTAB file");
            ptab_file.read_to_string(&mut ptab_contents).expect("Failed to read PTAB file");

            // Call the new method to create a Ptab instance
            let ptab = ptab::Ptab::new(&ptab_contents).expect("Failed to parse PTAB JSON");

            // Create an Ftab instance
            let mut ftab = ftab::Ftab::new();

            // Apply the PTAB data to Ftab
            ftab.apply(&ptab);

            // Convert the Ftab to bytes
            let bytes = ftab.to_bytes();

            // Write the bytes to the output file
            let mut output_file = File::create(&cli.output).expect("Failed to create output file");
            output_file.write_all(&bytes).expect("Failed to write to output file");

            println!("Flash table successfully generated at: {}", cli.output.display());
        }
        None => {
            println!("No subcommand specified. Use `--help` to see available options.");
        }
    }
}
