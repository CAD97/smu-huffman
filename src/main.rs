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
    if args.len() != 4 {
        print_usage();
    }

    let _exe_name = args.next().unwrap();
    match &*args.next().unwrap() {
        "compress" => process(&args.next().unwrap(), &args.next().unwrap(), compress),
        "decompress" => process(&args.next().unwrap(), &args.next().unwrap(), decompress),
        "test" => {
            if &*args.next().unwrap() != "roundtrip" {
                print_usage();
            }
            test(&args.next().unwrap())
        },
        _ => print_usage(),
    }
}

fn print_usage() -> ! {
    println!("Usage:");
    println!("  smu-huffman compress <input path> <output path>");
    println!("  smu-huffman decompress <input path> <output path>");
    println!("  smu-huffman test roundtrip <input path>");
    println!("To read/write to stdin/stdout, use `-` as the path.");
    process::exit(1)
}

fn process(input_path: &str, output_path: &str, op: impl for<'a> FnOnce(&'a [u8]) -> Vec<u8>) -> io::Result<()> {
    let input_bytes = read(input_path)?;
    let output_bytes = op(&input_bytes);
    write(output_path, &output_bytes)
}

fn read(input_path: &str) -> io::Result<Vec<u8>> {
    if input_path == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        fs::read(input_path)
    }
}

fn write(output_path: &str, bytes: &[u8]) -> io::Result<()> {
    if output_path == "-" {
        io::stdout().write_all(&bytes)
    } else {
        fs::write(output_path, &bytes)
    }
}

fn test(input_path: &str) -> io::Result<()> {
    let input_bytes = read(input_path)?;
    let roundtrip_bytes = decompress(&compress(&input_bytes));
    if input_bytes == roundtrip_bytes {
        println!("passed");
    } else {
        println!("failed");
        process::exit(1);
    }
    Ok(())
}
