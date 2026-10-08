use clap::Parser;
use std::fs;

use anyhow::{Context, Result};
use goblin::pe::PE;
mod args;
mod calls;
mod instructions;

fn main() -> Result<()> {
    let args = args::Args::parse();

    // Read input exe
    let buffer = fs::read(&args.input)
        .with_context(|| format!("[!] Failed to open input file: {:?}", args.input))?;

    // Parse PE, this will early fail (which is good)
    let pe = PE::parse(&buffer).context("[!] Failed to parse PE")?;
    let patched_data = calls::patch_instructions(&pe, buffer.clone(), args.mode)?;

    fs::write(&args.output, patched_data)
        .with_context(|| format!("[!] Failed to write output file: {:?}", args.output))?;

    println!("[+] Success: Wrote patched binary to {:?}", args.output);
    Ok(())
}
