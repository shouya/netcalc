import * as wasm from "./netcalc_bg.wasm";
import { __wbg_set_wasm } from "./netcalc_bg.js";
__wbg_set_wasm(wasm);
export * from "./netcalc_bg.js";
