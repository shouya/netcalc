mod netcalc;

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn convert(sep: &str, s: &str) -> String {
  console_error_panic_hook::set_once();
  netcalc::convert("v4", sep, s).unwrap_or_else(|err| format!("{}", err))
}
