use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use clap::{App, Arg};

use nand2tetris::jack::{tokenize, Parser};

fn to_xml(p: impl AsRef<Path>) {
    let f = File::open(p.as_ref()).expect("Can't open file");
    let mut reader = BufReader::new(f);
    let mut s = String::new();
    let mut buf = Vec::<u8>::new();
    while reader.read_until(b'\n', &mut buf).unwrap() != 0 {
        s.push_str(&String::from_utf8_lossy(&buf));
        buf.clear();
    }
    let tokens = tokenize(s.chars()).unwrap();
    let mut parser = Parser::new(tokens.into_iter());
    let ast = parser.parse().unwrap();
    let xml = ast.to_xml();

    let dir = p.as_ref().parent().unwrap();
    let file_name = format!("{}.xml", p.as_ref().file_stem().unwrap().to_str().unwrap());
    let out_path = dir.join(file_name);
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writer.write_all(xml.as_bytes()).unwrap();
}

fn main() {
    let args = App::new("parser")
        .arg(
            Arg::with_name("INPUT")
                .help(".jack file or dir path containing .jack file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input_path = Path::new(args.value_of("INPUT").unwrap());
    if input_path.is_dir() {
        for path in input_path
            .read_dir()
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| p.is_file() && p.to_str().unwrap().ends_with("jack"))
        {
            to_xml(path);
        }
    } else if input_path.is_file() && input_path.to_str().unwrap().ends_with("jack") {
        to_xml(input_path);
    }
}
