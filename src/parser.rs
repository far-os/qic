use std::{env, iter, collections::HashMap};

mod datatype;
use self::datatype::*;

#[derive(Debug)]
pub struct Program {
  pub index: usize,
  pub list: Vec<Token>,
  pub magic: bool,
}

impl Program {
  pub fn new() -> Self {
    Self {
      index: 0,
      list: Vec::new(),
      magic: true,
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

      if proc_tok.starts_with("v$") {
        let val = proc_tok.strip_prefix("v$").unwrap();
        let Some((l, r)) = val.split_once('.') else { panic!("Couldn't find path in value substitute - use '.' for path seperation") };

        self.list.push(Token::PathSubst(l.to_string(), r.to_string()));
        self.list.extend(waiting.into_iter().rev());
        continue;
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
        "rept" => Token::ReptStart,
        "endrept" => Token::ReptEnd,
        other => Token::Label(other.to_string()),
      });

      self.list.extend(waiting.into_iter().rev());
    }
  }

  pub fn parse(&mut self) -> Vec<u8> {
    let mut ret = Vec::new(); //Vec::from(magic_n.to_le_bytes());
    let mut block: Option<String> = None;
    let mut repetit_save: Vec<u8> = Vec::new();
    let mut repetit_by: Option<usize> = None;

    let mut data: HashMap<String, HashMap<String, usize>> = HashMap::new();

    let mut value = 0usize;

    let mut magic = true;

    while self.index < self.list.len() {
      match &self.list[self.index] {
        Token::BlockStart => {
          if block.is_some() {
            panic!("Cannot nest type block");
          }

          // next token is label
          let Token::Label(ref nm) = self.list[self.index + 1] else { panic!("No name provided for block"); };
          data.insert(nm.clone(), HashMap::new());

          block = Some(nm.clone());
          self.index += 1;
        },
        Token::BlockEnd => {
          block = None;
        },
        Token::ReptStart => {
          // TODO make better
          if repetit_by.is_some() {
            panic!("Cannot nest type rept");
          }

          // next token is number
          let Token::Value(u) = self.list[self.index + 1] else { panic!("No repeat count provided"); };
          repetit_by = Some(u);
          (repetit_save, ret) = (ret, repetit_save); // this is very naive. so our temporary repetition buffer IS THE MAIN BUFFER. the rest of the main buffer is moved out of the way. any other way would make everything else more complicated.

          self.index += 1;
        },
        Token::ReptEnd => {
          let Some(u) = repetit_by else { panic!("Invalid repeat end") };
          for _ in 0..u {
            repetit_save.extend(ret.clone());
          }

          (repetit_save, ret) = (ret, repetit_save); // undo what we did earlier. see above as to why i hate this code
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

          let listing = nm.clone();
          data.entry(block.clone().unwrap()).and_modify(|n| { n.insert(nm.clone(), value); });
          ret.extend_from_slice(&value.to_le_bytes()[..(*i as usize / 8)]);

          // data lookahead
          for k in self.index..self.list.len() {
            if let Token::PathSubst(l, r) = &self.list[k] {
              if *l == block.clone().unwrap() && *r == listing {
                self.list[k] = Token::Value(*data.get(l).unwrap().get(r).unwrap());
              }
            }
          }
        },
        Token::Command(com) => {
          match com.as_str() {
            "align" => {
              let Token::Value(wdt) = self.list[self.index + 1] else { panic!("Expected align width") };
              ret.extend(iter::repeat(0).take(wdt - (ret.len() % wdt)));
              self.index += 1;
            },
            "nomagic" => self.magic = false,
            "magic" => self.magic = true,
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

