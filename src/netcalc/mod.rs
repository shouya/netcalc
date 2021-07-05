use failure::{bail, ensure};
use std::str::FromStr;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

#[derive(Clone, Debug)]
pub enum Tree {
  Sat,
  Unsat,
  Mixed(Box<Tree>, Box<Tree>),
}

#[derive(Clone, Copy, Debug)]
pub enum Bit {
  B0,
  B1,
}

use Bit::*;
use Tree::*;

#[derive(Clone, Debug)]
pub struct Bits(Vec<Bit>);

type Result<T> = std::result::Result<T, failure::Error>;

impl From<Bit> for u8 {
  fn from(b: Bit) -> u8 {
    match b {
      B0 => 0,
      B1 => 1,
    }
  }
}

impl From<u8> for Bit {
  fn from(b: u8) -> Bit {
    match b {
      0 => B0,
      1 => B1,
      _ => panic!("Cannot convert from any non-binary u8"),
    }
  }
}

impl Bits {
  pub fn empty() -> Self {
    Self(vec![])
  }

  pub fn from_u8(n: u8) -> Self {
    let mut out = vec![];
    let mut m = n.reverse_bits();
    for _ in 0..=7 {
      match m & 1 {
        0 => out.push(Bit::B0),
        1 => out.push(Bit::B1),
        _ => unreachable!("Unreachable"),
      }
      m >>= 1;
    }
    Self(out)
  }

  pub fn extend(&mut self, other: Bits) {
    self.0.extend(other.0)
  }
  pub fn push(&mut self, bit: Bit) {
    self.0.push(bit)
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn truncate(&mut self, n: usize) {
    self.0.truncate(n)
  }

  pub fn parse_v4_addr(s: &str) -> Result<Self> {
    let mut bits = Self::empty();
    for segment in s.split(".") {
      log(segment);
      let byte = u8::from_str(dbg!(&segment))?;
      bits.extend(Self::from_u8(byte));
    }
    ensure!(bits.len() == 32, "Invalid IPv4 Address");

    Ok(bits)
  }

  pub fn parse_v4_cidr(s: &str) -> Result<Self> {
    match s.split("/").collect::<Vec<_>>().as_slice() {
      [left, right] => {
        let mut addr = Self::parse_v4_addr(left)?;
        let len = u8::from_str(right)?;
        log(&format!("{:?}", addr));
        ensure!(len <= 32, "Invalid IPv4 CIDR prefix length");
        addr.truncate(len as usize);
        Ok(addr)
      }
      _ => {
        bail!("Invalid IPv4 CIDR")
      }
    }
  }

  pub fn parse_v4(s: &str) -> Result<Self> {
    Self::parse_v4_cidr(s).or_else(|_| Self::parse_v4_addr(s))
  }

  pub fn split(mut self) -> Result<(Bit, Self)> {
    ensure!(self.len() > 0, "Cannot split on empty bits");
    let tail = self.0.split_off(1);
    let head = self.0[0];
    Ok((head, Self(tail)))
  }

  pub fn to_u32(&self) -> Result<u32> {
    ensure!(
      self.len() == 32,
      "Cannot convert to u32 from bits of non-32 length"
    );

    let n = self
      .0
      .iter()
      .fold(0, |s, x| (s << 1) | (u8::from(*x) as u32));

    Ok(n)
  }

  pub fn to_v4_addr(&self) -> Result<String> {
    ensure!(self.len() == 32, "Invalid prefix length");
    let i = self.to_u32()?;
    let (a, b, c, d) = (i >> 24 & 0xFF, i >> 16 & 0xFF, i >> 8 & 0xFF, i & 0xFF);
    Ok(format!("{}.{}.{}.{}", a, b, c, d))
  }

  pub fn to_v4_cidr(&self) -> Result<String> {
    let len = self.len();
    ensure!(len <= 32, "Invalid prefix length");

    let mut bits = self.clone();
    bits.right_pad(32, B0);
    Ok(format!("{}/{}", bits.to_v4_addr()?, len))
  }

  fn right_pad(&mut self, new_len: usize, bit: Bit) {
    self.0.resize(new_len, bit)
  }
}

impl Tree {
  pub fn new() -> Self {
    Unsat
  }

  pub fn flip(self) -> Self {
    match self {
      Sat => Unsat,
      Unsat => Sat,
      Mixed(l, r) => Mixed(Box::new(l.flip()), Box::new(r.flip())),
    }
  }

  pub fn add_cidr(self, cidr: Bits) -> Self {
    if cidr.len() == 0 {
      return Sat;
    }
    let (h, t) = cidr.split().unwrap();
    match (self, h) {
      (Sat, _) => Sat,
      (Unsat, B0) => Mixed(Box::new(Unsat.add_cidr(t)), Box::new(Unsat)),
      (Unsat, B1) => Mixed(Box::new(Unsat), Box::new(Unsat.add_cidr(t))),
      (Mixed(l, r), B0) => Mixed(Box::new(l.add_cidr(t)), r),
      (Mixed(l, r), B1) => Mixed(l, Box::new(r.add_cidr(t))),
    }
  }

  pub fn del_cidr(self, cidr: Bits) -> Self {
    self.flip().add_cidr(cidr).flip()
  }

  pub fn optimize(self) -> Self {
    match self {
      Sat => Sat,
      Unsat => Unsat,
      Mixed(l, r) => match (l.optimize(), r.optimize()) {
        (Sat, Sat) => Sat,
        (Unsat, Unsat) => Unsat,
        (ol, or) => Mixed(Box::new(ol), Box::new(or)),
      },
    }
  }

  pub fn prefixes(&self) -> Vec<Bits> {
    self.clone().optimize().prefixes_priv(Bits::empty())
  }

  fn prefixes_priv(self, prefix: Bits) -> Vec<Bits> {
    match self {
      Sat => vec![prefix],
      Unsat => vec![],
      Mixed(l, r) => {
        let mut l_prefix = prefix.clone();
        let mut r_prefix = prefix.clone();
        l_prefix.push(B0);
        r_prefix.push(B1);

        let l_iter = l.prefixes_priv(l_prefix);
        let r_iter = r.prefixes_priv(r_prefix);
        l_iter.into_iter().chain(r_iter).collect()
      }
    }
  }
}

pub fn convert(sep: &str, s: &str) -> Result<String> {
  let instructions = Instruction::parse_lines(s)?;
  let tree = exec_instructions(instructions.as_slice());
  let cidrs = tree
    .prefixes()
    .into_iter()
    .map(|x| x.to_v4_cidr())
    .collect::<Result<Vec<_>>>()?
    .as_slice()
    .join(sep)
    .into();
  Ok(cidrs)
}

enum Instruction {
  Add(Bits),
  Del(Bits),
}

use Instruction::*;

fn exec_instructions(instructions: &[Instruction]) -> Tree {
  let mut t = Tree::new();
  for inst in instructions {
    match inst {
      Add(p) => t = t.add_cidr(p.clone()),
      Del(p) => t = t.del_cidr(p.clone()),
    }
  }
  t
}

impl Instruction {
  fn parse_lines(s: &str) -> Result<Vec<Instruction>> {
    use Instruction::*;

    let mut out = vec![];
    for line in s.lines() {
      let line = line.trim();
      match &line[..1] {
        "#" => (),
        "+" => out.push(Add(Bits::parse_v4(&line[1..])?)),
        "-" => out.push(Del(Bits::parse_v4(&line[1..])?)),
        "" => (),
        _ => bail!("Unrecognized line: {}", line),
      }
    }

    Ok(out)
  }
}
