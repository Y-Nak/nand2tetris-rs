use std::path::Path;

use clap::{App, Arg};

use nand2tetris::AsmGenerator;

fn main() {
    let args = App::new("code_gen")
        .arg(
            Arg::with_name("INPUT")
                .help("VM file or dir path")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("out")
                .short("o")
                .help("Output asm file path")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let mut gen = AsmGenerator::new();
    let input_path = Path::new(args.value_of("INPUT").unwrap());
    if input_path.is_dir() {
        for path in input_path.read_dir().unwrap().filter(|p| {
            let p = p.as_ref().unwrap().path();
            p.is_file() && p.to_str().unwrap().ends_with("vm")
        }) {
            gen.gen(path.unwrap().path()).unwrap();
        }
    } else {
        println!("{:?}", input_path);
        if !input_path.to_str().unwrap().ends_with("vm") {
            panic!("vm file is expected");
        }
        gen.gen(input_path).unwrap();
    }

    let default_out = format! {"{}.asm", input_path.file_stem().unwrap().to_str().unwrap()};
    let out_path = args.value_of("out").unwrap_or(&default_out);
    gen.flush(out_path).unwrap();
}
