use std::str::FromStr;
use std::{convert::TryInto, marker::PhantomData};

use failure::{bail, ensure};

mod alg;

pub use alg::{Bit, Prefix, Tree};

type Result<T> = std::result::Result<T, failure::Error>;

trait AddrType {
  fn parse_addr(s: &str) -> Result<Prefix>;
  fn parse_cidr(s: &str) -> Result<Prefix>;
  fn parse_range(s: &str) -> Result<Tree>;
  fn cidr_from_prefix(p: Prefix) -> Result<String>;
}

struct V4;

impl AddrType for V4 {
  fn parse_addr(s: &str) -> Result<Prefix> {
    let mut prefix = Prefix::empty();
    for segment in s.split('.') {
      let byte = u8::from_str(segment)?;
      prefix.extend(Prefix::from_u8(byte));
    }
    ensure!(prefix.len() == 32, "Invalid IPv4 Address");

    Ok(prefix)
  }

  fn parse_cidr(s: &str) -> Result<Prefix> {
    match s.split('/').collect::<Vec<_>>().as_slice() {
      [left, right] => {
        let mut addr = Self::parse_addr(left)?;
        let len = u8::from_str(right)?;
        ensure!(len <= 32, "Invalid IPv4 CIDR prefix length");
        addr.truncate(len as usize);
        Ok(addr)
      }
      _ => bail!("Invalid IPv4 CIDR"),
    }
  }

  fn parse_range(s: &str) -> Result<Tree> {
    match s.split('-').collect::<Vec<_>>().as_slice() {
      [left, right] => {
        let left = Self::parse_addr(left)?;
        let right = Self::parse_addr(right)?;
        Ok(Tree::from_range(&left, &right)?)
      }
      _ => bail!("Invalid IPv4 range"),
    }
  }

  fn cidr_from_prefix(mut prefix: Prefix) -> Result<String> {
    let len = prefix.len();
    ensure!(len <= 32, "Invalid prefix length");

    prefix.right_pad(32, Bit::B0);
    let chunks = prefix.chunks(8)?;
    let [a, b, c, d]: [_; 4] = chunks.as_slice().try_into()?;
    Ok(format!("{}.{}.{}.{}/{}", a, b, c, d, len))
  }
}

#[allow(unused)]
struct V6;

struct App<T>(PhantomData<T>);

enum Operand<T> {
  #[allow(unused)]
  Unused(T),
  Prefix(Prefix),
  Tree(Tree),
}

impl<T> Operand<T> {
  fn parse(s: &str) -> Result<Self>
  where
    T: AddrType,
  {
    T::parse_addr(s)
      .map(Operand::Prefix)
      .or_else(|_| T::parse_cidr(s).map(Operand::Prefix))
      .or_else(|_| T::parse_range(s).map(Operand::Tree))
  }
}

enum TreeOp<T> {
  Add(Operand<T>),
  Del(Operand<T>),
  Noop,
}

impl<T> TreeOp<T> {
  fn parse(s: &str) -> Result<Self>
  where
    T: AddrType,
  {
    // otherwise the [..1] will panic
    if s.is_empty() {
      return Ok(TreeOp::Noop);
    }

    match &s[..1] {
      "+" => Ok(TreeOp::Add(Operand::parse(&s[1..])?)),
      "-" => Ok(TreeOp::Del(Operand::parse(&s[1..])?)),
      "#" => Ok(TreeOp::Noop),
      // empty line
      "" => Ok(TreeOp::Noop),
      _ => bail!("Unrecognized line: {}", s),
    }
  }

  fn apply(self, tree: Tree) -> Tree {
    match self {
      TreeOp::Add(Operand::Prefix(p)) => tree.add(p),
      TreeOp::Del(Operand::Prefix(p)) => tree.del(p),
      TreeOp::Add(Operand::Tree(o)) => tree.add_tree(o),
      TreeOp::Del(Operand::Tree(o)) => tree.del_tree(o),
      TreeOp::Noop => tree,
      _ => unreachable!(),
    }
  }
}

impl<T: AddrType> App<T> {
  fn convert(sep: &str, s: &str) -> Result<String> {
    let mut tree = Tree::new();

    for line in s.lines() {
      let line = line.trim();
      let op: TreeOp<T> = TreeOp::parse(line)?;
      tree = op.apply(tree);
    }

    let cidrs = tree
      .prefixes()
      .into_iter()
      .map(T::cidr_from_prefix)
      .collect::<Result<Vec<_>>>()?
      .join(sep);

    Ok(cidrs)
  }
}

#[allow(unused)]
pub fn convert(version: &str, sep: &str, s: &str) -> Result<String> {
  match version {
    "4" => App::<V4>::convert(sep, s),
    // "6" => App::<V6>::convert(sep, s),
    _ => bail!("Unrecognized version: {}", version),
  }
}
