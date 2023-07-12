use failure::ensure;
use itertools::{EitherOrBoth, Itertools};
use std::cmp::Ordering;

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

  pub fn chunks(&self, n: usize) -> Result<Vec<u64>> {
    ensure!(n <= u64::BITS as usize, "cannot chunk larger than u64");
    ensure!(0 < n, "cannot chunk by 0");

    let mut out = vec![];

    for chunk in self.0.chunks(n) {
      ensure!(chunk.len() == n, "cannot chunk by non-uniform size");
      out.push(Bits(chunk.to_vec()).to_u64()?)
    }

    Ok(out)
  }

  pub fn to_u64(&self) -> Result<u64> {
    ensure!(
      self.len() <= u64::BITS as usize,
      "Cannot convert to u64 from bits of length > 64"
    );

    let mut out = 0;
    for bit in self.0.iter().rev() {
      out <<= 1;
      out |= u8::from(*bit) as u64;
    }
    Ok(out)
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

  pub fn split(mut self) -> Result<(Bit, Self)> {
    ensure!(self.len() > 0, "Cannot split on empty bits");
    let tail = self.0.split_off(1);
    let head = self.0[0];
    Ok((head, Self(tail)))
  }

  pub fn right_pad(&mut self, new_len: usize, bit: Bit) {
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
    Self::from_range_at(Bits::empty(), start, end)
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

  pub fn del(self, cidr: Bits) -> Self {
    self.flip().add(cidr).flip()
  }

  pub fn add_tree(self, tree: Tree) -> Self {
    self.union(tree)
  }

  pub fn del_tree(self, _tree: Tree) -> Self {
    todo!()
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