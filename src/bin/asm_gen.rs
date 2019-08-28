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
        .arg(
            Arg::with_name("no-init")
                .help("Compile VM codes in project07 and in first half of project08")
                .long("no-init"),
        )
        .get_matches();

    let no_init = args.occurrences_of("no-init") > 0;
    let mut gen = AsmGenerator::new(no_init);
    let input_path = Path::new(args.value_of("INPUT").unwrap());
    if input_path.is_dir() {
        for path in input_path
            .read_dir()
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| p.is_file() && p.to_str().unwrap().ends_with("vm"))
        {
            gen.gen(path).unwrap();
        }
    } else {
        if !input_path.to_str().unwrap().ends_with("vm") {
            panic!("vm file is expected");
        }
        gen.gen(input_path).unwrap();
    }

    let default_out = format! {"{}.asm", input_path.file_stem().unwrap().to_str().unwrap()};
    let out_path = args.value_of("out").unwrap_or(&default_out);
    gen.flush(out_path).unwrap();
}
