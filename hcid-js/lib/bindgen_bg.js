
const bytes = require('fs').readFileSync(__dirname + '/bindgen_bg.wasm');
let imports = {};
imports['./bindgen'] = require('./bindgen');

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
module.exports = wasmInstance.exports;
