import * as wasm from "netcalc";

window.conv = function () {
  const input = document.querySelector("#rules textarea").value;
  document.querySelector("#results textarea").value = wasm.convert("\n", input);
};

document.onready = window.conv;
