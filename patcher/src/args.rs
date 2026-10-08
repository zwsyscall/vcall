use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the input PE file (e.g., loader.exe)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Path to the output file
    #[arg(short, long, default_value = "patched.exe")]
    pub output: PathBuf,

    /// Mode for handling calls
    #[arg(short, long, value_enum, default_value_t = CallTarget::Encrypted)]
    pub mode: CallTarget,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CallTarget {
    Encrypted,
    Unencrypted,
}

impl CallTarget {
    pub fn encrypted(&self) -> bool {
        match self {
            Self::Encrypted => true,
            Self::Unencrypted => false,
        }
    }
}
