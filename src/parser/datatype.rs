#[derive(Debug)]
pub enum Op {
  Add,
  Sub,
  Mul,
  Div,
}

#[derive(Debug)]
pub enum Token {
  BlockStart,
  BlockEnd,
  Integer(u8),
  Label(String),
  Assign, // :
  Value(usize),
  Command(String),
  Operation(Op),
  BracketOpen,
  BracketClose,
  EndLn,
  PathSubst(String, String),
  ReptStart,
  ReptEnd,
}
