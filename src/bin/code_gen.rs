use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use clap::{App, Arg};

use nand2tetris::gen_code;

fn main() {
    let args = App::new("code_gen")
        .arg(
            Arg::with_name("INPUT")
                .help("Asm file path")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("out")
                .short("o")
                .help("Output binary file path")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let asm_path = Path::new(args.value_of("INPUT").unwrap());
    let strm = BufReader::new(
        File::open(asm_path).expect(&format! {"Can't open asm file: {:?}", asm_path}),
    );
    let code = gen_code(strm).unwrap();

    let default_out = format! {"{}.hack", asm_path.file_stem().unwrap().to_str().unwrap()};
    let out_path = args.value_of("out").unwrap_or(&default_out);
    let mut writer = BufWriter::new(
        File::create(out_path).expect(&format! {"Can't open output file: {:?}", out_path}),
    );
    for line in code {
        writer.write_all(line.as_bytes()).unwrap();
        writer.write_all("\n".as_bytes()).unwrap();
    }
}
