use std::{env, fs::File, io::prelude::*};

mod parser;
use parser::Program;

const IN_EXT: &str = "qit";
const OUT_EXT: &str = "qi";

const MAGIC: u32 = 0xc091fa2b; // magic number for qi format

fn main() {
  let argv = env::args().collect::<Vec<String>>();

  if argv.len() <= 1 {
    eprintln!("Please specify an input file.");
    return;
  }

  let Ok(mut file) = File::open(argv[1].clone()) else {
    eprintln!("File not found {}", argv[1]);
    return;
  };

  let (fileNm, ext) = argv[1].rsplit_once('.').unwrap_or((argv[1].as_str(), ""));

  if ext != IN_EXT {
    eprintln!("Warning, input file of extention \"{ext}\", may not be a \"{IN_EXT}\" file");
  }
  
  let mut input = String::new();

  file.read_to_string(&mut input).expect("Can't read String"); // if the string isn't there for some reason
  let feed = input.trim_end().split_whitespace().collect::<Vec<_>>(); // convert buffer to string
  
  let mut runner = Program::new();
  runner.lexer(feed);

  let data = runner.parse(MAGIC);

  let Ok(mut out_f) = File::create(format!("{fileNm}.{OUT_EXT}")) else {
    eprintln!("Unable to create file {fileNm}.{OUT_EXT}");
    return;
  };

  out_f.write(&data).expect("Could not write to file");
}
