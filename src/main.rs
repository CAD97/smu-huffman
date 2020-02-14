use {
    smu_huffman::{compress, decompress},
    std::{
        env, fs,
        io::{self, prelude::*},
        process,
    },
};

fn main() -> io::Result<()> {
    let mut args = env::args();

    // Usage
    if args.len() != 3 {
        print_usage();
    }

    let _exe_name = args.next().unwrap();
    let output = match &*args.next().unwrap() {
        "compress" => process(&args.next().unwrap(), compress),
        "decompress" => process(&args.next().unwrap(), decompress),
        _ => print_usage(),
    }?;

    io::stdout().write_all(&output)
}

fn print_usage() -> ! {
    println!("Usage:");
    println!("  smu-huffman compress <path>           Compress a file and write it to stdout");
    println!("  smu-huffman compress -                Compress stdin and write it to stdout");
    println!("  smu-huffman decompress <path>         Decompress a file and write it to stdout");
    println!("  smu-huffman decompress -              Decompress stdin and write it to stdout");
    process::exit(1)
}

fn process(path: &str, op: impl for<'a> FnOnce(&'a [u8]) -> Vec<u8>) -> io::Result<Vec<u8>> {
    let buf = if path == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        fs::read(path)?
    };
    Ok(op(&buf))
}
