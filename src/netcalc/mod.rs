use failure::{bail, ensure};
use itertools::{EitherOrBoth, Itertools};
use std::{cmp::Ordering, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bit {
  B0,
  B1,
}

use Bit::*;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bits(Vec<Bit>);

impl From<&[u8]> for Bits {
  fn from(bits: &[u8]) -> Self {
    Self(bits.iter().map(|b| Bit::from(*b)).collect())
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

  pub fn append(&self, bit: Bit) -> Self {
    let mut out = self.clone();
    out.push(bit);
    out
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
    let (a, b, c, d) =
      (i >> 24 & 0xFF, i >> 16 & 0xFF, i >> 8 & 0xFF, i & 0xFF);
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

impl PartialOrd for Bits {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    use EitherOrBoth::{Both, Left, Right};
    use Ordering::{Equal, Greater, Less};

    for x in self.0.iter().zip_longest(other.0.iter()) {
      match x {
        Both(B0, B0) => continue,
        Both(B1, B1) => continue,
        Both(B0, B1) => return Some(Less),
        Both(B1, B0) => return Some(Greater),
        Right(_) => return None,
        Left(_) => return None,
      }
    }

    Some(Equal)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
  Sat,
  Unsat,
  Mixed(Box<Tree>, Box<Tree>),
}
use Tree::*;

impl Tree {
  pub fn new() -> Self {
    Unsat
  }

  pub fn from_range(start: &Bits, end: &Bits) -> Self {
    Self::from_range_at(Bits::empty(), &start, &end)
  }

  fn from_range_at(curr: Bits, start: &Bits, end: &Bits) -> Self {
    if &curr < start || &curr > end {
      return Unsat;
    }

    if start <= &curr && &curr <= end {
      return Sat;
    }

    let left = Self::from_range_at(curr.append(B0), start, end);
    let right = Self::from_range_at(curr.append(B1), start, end);

    Mixed(Box::new(left.optimize()), Box::new(right.optimize()))
  }

  pub fn flip(self) -> Self {
    match self {
      Sat => Unsat,
      Unsat => Sat,
      Mixed(l, r) => Mixed(Box::new(l.flip()), Box::new(r.flip())),
    }
  }

  pub fn union(self, other: Self) -> Self {
    match (self, other) {
      (Sat, _) => Sat,
      (_, Sat) => Sat,
      (Unsat, Unsat) => Unsat,
      (Unsat, Mixed(l, r)) => {
        Mixed(Box::new(Unsat.union(*l)), Box::new(Unsat.union(*r)))
      }
      (Mixed(l, r), Unsat) => {
        Mixed(Box::new(l.union(Unsat)), Box::new(r.union(Unsat)))
      }
      (Mixed(l1, r1), Mixed(l2, r2)) => {
        Mixed(Box::new(l1.union(*l2)), Box::new(r1.union(*r2)))
      }
    }
  }

  pub fn intersect(self, other: Self) -> Self {
    match (self, other) {
      (Sat, Sat) => Sat,
      (_, Unsat) => Unsat,
      (Unsat, _) => Unsat,
      (Mixed(l, r), Sat) => {
        Mixed(Box::new(l.intersect(Sat)), Box::new(r.intersect(Sat)))
      }
      (Sat, Mixed(l, r)) => {
        Mixed(Box::new(Sat.intersect(*l)), Box::new(Sat.intersect(*r)))
      }
      (Mixed(l1, r1), Mixed(l2, r2)) => {
        Mixed(Box::new(l1.intersect(*l2)), Box::new(r1.intersect(*r2)))
      }
    }
  }

  pub fn add(self, cidr: Bits) -> Self {
    if cidr.len() == 0 {
      return Sat;
    }
    let (h, t) = cidr.split().unwrap();
    match (self, h) {
      (Sat, _) => Sat,
      (Unsat, B0) => Mixed(Box::new(Unsat.add(t)), Box::new(Unsat)),
      (Unsat, B1) => Mixed(Box::new(Unsat), Box::new(Unsat.add(t))),
      (Mixed(l, r), B0) => Mixed(Box::new(l.add(t)), r),
      (Mixed(l, r), B1) => Mixed(l, Box::new(r.add(t))),
    }
  }

  pub fn del_cidr(self, cidr: Bits) -> Self {
    self.flip().add(cidr).flip()
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
        let mut r_prefix = prefix;
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

enum BitsOrTree {
  Bits(Bits),
  Tree(Tree),
}

impl BitsOrTree {
  fn parse(_s: &str) -> Result<Self> {
    todo!()
  }
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
      Add(p) => t = t.add(p.clone()),
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

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_from_range() {
    let start = Bits::from(&[0, 0, 0, 1u8][..]);
    let end = Bits::from(&[0, 1, 1, 0u8][..]);

    let expected = Tree::new()
      .add([0, 0, 0, 1u8][..].into())
      .add([0, 0, 1, 0u8][..].into())
      .add([0, 0, 1, 1u8][..].into())
      .add([0, 1, 0, 0u8][..].into())
      .add([0, 1, 0, 1u8][..].into())
      .add([0, 1, 1, 0u8][..].into())
      .optimize();

    let actual = Tree::from_range(&start, &end);

    assert_eq!(expected, actual);
  }
}
