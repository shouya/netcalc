import { convert } from "./netcalc.js";

async function conv() {
  const input = document.querySelector("#rules textarea").value;
  let sep = document.querySelector("#separator").value;
  if (sep == "\\n") {
    sep = "\n";
  }
  const output = convert(sep, input);
  document.querySelector("#results textarea").value = output;
}

conv();
