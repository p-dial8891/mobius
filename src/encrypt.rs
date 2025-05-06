use simple_crypt::encrypt_file;
use std::path::Path;
use clap::Parser;


#[derive(Parser)]
struct Flags {
    /// Sets the server address to connect to.
    #[arg(long)]
    key: String
}

fn main() {
	let flags = Flags::parse();
	encrypt_file(Path::new("input.json"), Path::new("mobius.encrypted"), flags.key.as_bytes()).expect("Failed to encrypt the file");
}
