use std::{env, iter};

mod datatype;
use self::datatype::*;

#[derive(Debug)]
pub struct Program {
  pub index: usize,
  pub list: Vec<Token>,
}

impl Program {
  pub fn new() -> Self {
    Self {
      index: 0,
      list: Vec::new(),
    }
  }

  pub fn lexer(&mut self, item: Vec<&str>) {
    let mut in_com = false;
    for tok in item.iter() {
      if *tok == "##" { in_com = !in_com; continue; }
      if in_com { continue; }

      let mut proc_tok = tok.clone();
      if proc_tok.starts_with('[') {
        self.list.push(Token::BracketOpen);
        proc_tok = &proc_tok[1..];
      }

      let mut waiting = Vec::new();
      if tok.ends_with(';') {
        waiting.push(Token::EndLn);
        proc_tok = &proc_tok[..proc_tok.len()-1];
      }

      if proc_tok.ends_with(']') {
        waiting.push(Token::BracketClose);
        proc_tok = &proc_tok[..proc_tok.len()-1];
      }

      let mut extr_env = String::from(proc_tok);

      if proc_tok.starts_with("e$") {
        if let Ok(ev) = env::var(proc_tok.strip_prefix("e$").unwrap()) {
          extr_env = ev;
        }
      }

      proc_tok = extr_env.as_str();

      self.list.push(match proc_tok {
        "block" => Token::BlockStart,
        "endblock" => Token::BlockEnd,
        inttype if inttype.starts_with("int") => Token::Integer(inttype[3..].parse::<u8>().unwrap()),
        special_com if special_com.starts_with('!') => Token::Command(special_com[1..].to_string()),
        ":" => Token::Assign,
        ";" => Token::EndLn,
        hn if hn.starts_with("0x") => Token::Value(usize::from_str_radix(hn.strip_prefix("0x").unwrap(), 16).unwrap()),
        n if n.starts_with(char::is_numeric) => Token::Value(n.parse::<usize>().unwrap()),
        "+" => Token::Operation(Op::Add),
        "-" => Token::Operation(Op::Sub),
        "*" => Token::Operation(Op::Mul),
        "/" => Token::Operation(Op::Div),
        other => Token::Label(other.to_string()),
      });

      self.list.extend(waiting.into_iter().rev());
    }
  }

  pub fn parse(&mut self, magic_n: u32) -> Vec<u8> {
    let mut ret = Vec::from(magic_n.to_le_bytes());
    let mut in_block = false;

    let mut value = 0usize;

    while self.index < self.list.len() {
      match &self.list[self.index] {
        Token::BlockStart => {
          in_block = true;
          // next token is label
          let Token::Label(ref nm) = self.list[self.index + 1] else { panic!("No name provided for block"); };
          self.index += 1;
        },
        Token::BlockEnd => {
          in_block = false;
        },
        Token::Integer(i) => {
          if *i % 8 != 0 || *i < 1 || *i > 64 { panic!("Invalid integer width {i}"); };

          // name ignored
          let Token::Label(ref nm) = self.list[self.index + 1] else { panic!("No name provided for statement"); };
          let Token::Assign = self.list[self.index + 2] else { panic!("No assign token found"); };
          
          match self.list[self.index + 3] {
            Token::Value(v) => { value = v; self.index += 3; },
            Token::BracketOpen => {
              let Token::Value(lhs) = self.list[self.index + 4] else { panic!("Expected number") };
              let Token::Operation(ref op) = self.list[self.index + 5] else { panic!("Expected operation") };
              let Token::Value(rhs) = self.list[self.index + 6] else { panic!("Expected number") };
              
              value = match op {
                Op::Add => lhs + rhs,
                Op::Sub => lhs - rhs,
                Op::Mul => lhs * rhs,
                Op::Div => lhs / rhs,
              };

              let Token::BracketClose = self.list[self.index + 7] else { panic!("No closing bracket found"); };
              self.index += 7;
            },
            _ => panic!("hat"),
          }

          let Token::EndLn = self.list[self.index + 1] else { panic!("No semicolon found"); };
          self.index += 1;

          ret.extend_from_slice(&value.to_le_bytes()[..(*i as usize / 8)]);
        },
        Token::Command(com) => {
          match com.as_str() {
            "align" => {
              let Token::Value(wdt) = self.list[self.index + 1] else { panic!("Expected align width") };
              ret.extend(iter::repeat(0).take(wdt - (ret.len() % wdt)));
              self.index += 1;
            }
            _ => panic!("Unknown command {com}"),
          }
        }
        a => panic!("what {:?}", a),
      }

      self.index += 1;
    }

    ret
  }
}

