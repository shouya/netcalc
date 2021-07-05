use wasm_bindgen::prelude::*;

type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
  netcalc::convert(sep, s).unwrap_or_else(|err| format!("{}", err))
}
