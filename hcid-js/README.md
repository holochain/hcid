# HCID-JS

Generate Holochain IDs in javascript. A thin wrapper around rust compiled to WASM

## Usage

This module exports a class called `Encoding` which can be used to construct an encoding for the different types of Holochain identifiers. Each of these identifiers are given a three character prefix:
- AgentID (from signing key) : 'hcs'
- ...

Depending on if you are using the module in node.js or the browser the calling syntax is slightly different. This is because in the browser the WASM must be asynchronously compiled and instantiated to prevent blocking the main thread. As a result all of the calls return promises in the browser but not in node.

### Node.js
```javascript

const publicKey = [...] // UInt8Array of bytes of public key

const enc = new Encoding('hcs')
const agentId = enc.encode(publicKey)
const restoredPublicKey = enc.decode(id)
```

### browser
```javascript

const publicKey = [...] // UInt8Array of bytes of public key

const enc = await new Encoding('hcs')
const agentId = await enc.encode(publicKey)
const restoredPublicKey = await enc.decode(id)
```