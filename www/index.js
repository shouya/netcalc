const { convert } = wasm_bindgen;

async function conv() {
  await wasm_bindgen("./netcalc_bg.wasm");

  const input = document.querySelector("#rules textarea").value;
  let sep = document.querySelector("#separator").value;
  if (sep == "\\n") {
    sep = "\n";
  }
  const output = convert("v4", sep, input);
  document.querySelector("#results textarea").value = output;
}

conv();
