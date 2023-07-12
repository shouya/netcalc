mod netcalc;
mod utils;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn convert(sep: &str, s: &str) -> String {
  netcalc::convert("4", sep, s).unwrap_or_else(|err| format!("{}", err))
}
