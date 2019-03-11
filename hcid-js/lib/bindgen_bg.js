
const path = require('path').join(__dirname, 'bindgen_bg.wasm');
const bytes = require('fs').readFileSync(path);
let imports = {};
imports['./bindgen'] = require('./bindgen');

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
module.exports = wasmInstance.exports;
