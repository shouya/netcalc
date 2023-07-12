import * as wasm_bindgen from "/netcalc.js";

async function conv() {
  await wasm_bindgen();

  const input = document.querySelector("#rules textarea").value;
  let sep = document.querySelector("#separator").value;
  if (sep == "\\n") {
    sep = "\n";
  }
  const output = wasm_bindgen.convert(sep, input);
  document.querySelector("#results textarea").value = output;
}

conv();
