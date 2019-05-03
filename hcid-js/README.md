# HCID-JS

Generate Holochain IDs in javascript. A thin wrapper around rust compiled to WASM

## Installation

This package is distributed via npm and can be installed using

```
npm install @holochain/hcid-js
```

## Usage

This module exports a class called `Encoding` which can be used to construct an encoding for the different types of Holochain identifiers. Each of these identifiers are given a three character prefix:
- AgentID (from signing key) : 'hcs'
- ...

Depending on if you are using the module in node.js or the browser the calling syntax is slightly different. This is because in the browser the WASM must be asynchronously compiled and instantiated to prevent blocking the main thread. As a result all of the constructor returns a promises in the browser but not in node.

```javascript

const publicKey = [...] // UInt8Array of bytes of public key

const enc = new Encoding('hcs0') // node.js
const enc = await new Encoding('hcs0') // browser

const agentId = enc.encode(publicKey)
const restoredPublicKey = enc.decode(id)
```

## Building

From the root of the repo (hcid) the package can be build using
```
make build
```

and tests run using 

```
make test
```

Note this runs browser tests which may fail if you do not have both firefox and chrome installed. On linux set the environment variable `CHROME_BIN=chromium`.


