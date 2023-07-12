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
    for bit in self.0.iter() {
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

  pub fn mixed(l: Self, r: Self) -> Self {
    Mixed(Box::new(l), Box::new(r))
  }

  pub fn from_range(start: &Bits, end: &Bits) -> Result<Self> {
    ensure!(start <= end, "In a range, start must be <= end");
    Ok(Self::from_range_at(Bits::empty(), start, end))
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
      // sat or unsat on the right
      (_, Sat) => Sat,
      (a, Unsat) => a,
      // sat or unsat on the left
      (Sat, _) => Sat,
      (Unsat, b) => b,
      // both mixed
      (Mixed(l1, r1), Mixed(l2, r2)) => {
        let l = l1.union(*l2);
        let r = r1.union(*r2);
        Self::mixed(l, r).optimize()
      }
    }
  }

  pub fn difference(self, other: Self) -> Self {
    match (self, other) {
      // sat or unsat on the right
      (_, Sat) => Unsat,
      (a, Unsat) => a,
      // sat or unsat on the left
      (Sat, b) => b.flip(),
      (Unsat, _) => Unsat,
      // both mixed
      (Mixed(a0, a1), Mixed(b0, b1)) => {
        let l = a0.difference(*b0);
        let r = a1.difference(*b1);
        Self::mixed(l, r).optimize()
      }
    }
  }

  pub fn add(self, bits: Bits) -> Self {
    if bits.len() == 0 {
      return Sat;
    }
    let (h, t) = bits.split().unwrap();
    match (self, h) {
      (Sat, _) => Sat,
      (Unsat, B0) => Mixed(Box::new(Unsat.add(t)), Box::new(Unsat)),
      (Unsat, B1) => Mixed(Box::new(Unsat), Box::new(Unsat.add(t))),
      (Mixed(l, r), B0) => Mixed(Box::new(l.add(t)), r),
      (Mixed(l, r), B1) => Mixed(l, Box::new(r.add(t))),
    }
  }

  pub fn del(self, bits: Bits) -> Self {
    self.flip().add(bits).flip()
  }

  pub fn add_tree(self, tree: Tree) -> Self {
    self.union(tree)
  }

  pub fn del_tree(self, tree: Tree) -> Self {
    self.difference(tree)
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

    let actual = Tree::from_range(&start, &end).unwrap();

    assert_eq!(expected, actual);
  }

  #[test]
  fn test_chunk() {
    let mut bits = Bits::from_u8(1);
    bits.extend(Bits::from_u8(2));
    bits.extend(Bits::from_u8(3));
    bits.extend(Bits::from_u8(4));

    assert_eq!(bits.chunks(8).unwrap(), vec![1, 2, 3, 4]);
  }
}
