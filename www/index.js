import * as wasm from "netcalc";

window.conv = function () {
  const input = document.querySelector("#rules textarea").value;
  let sep = document.querySelector("#separator").value;
  if (sep == "\\n") {
    sep = "\n";
  }
  const output = wasm.convert(sep, input);
  document.querySelector("#results textarea").value = output;
};

window.conv();
