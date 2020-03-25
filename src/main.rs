use std::env;
use std::path::Path;
use std::time::Instant;

mod args;
mod stream_utils;
mod unpacker;
mod zip;

use args::input_parser;

fn main() -> std::io::Result<()> {
    let args = input_parser::parse_args(&env::args().collect()).unwrap();

    println!("\nSource ZIP: {}", args.in_file);
    println!("Output dir: {}\n", args.out_folder);

    let out_folder = Path::new(&args.out_folder);

    if out_folder.exists() && !out_folder.is_dir() {
        panic!("Output dir is not a dir.");
    } else if out_folder.exists() {
        println!("Cleaning up the mess...");
        std::fs::remove_dir_all(out_folder)?;
    }

    println!("Unpacking...\n");

    let start_time = Instant::now();

    std::fs::create_dir(out_folder)?;

    unpacker::unpack_archive(Path::new(&args.in_file), out_folder)?;

    println!(
        "Time spent: {} sec",
        Instant::now().duration_since(start_time).as_secs()
    );

    Ok(())
}
