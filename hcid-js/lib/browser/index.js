/* eslint camelcase:0 */

const bindgen = require('./bindgen')

function txError (fn) {
  try {
    return fn()
  } catch (e) {
    throw new Error(e)
  }
}

function checkFixBuffer (buf) {
  if (typeof Buffer === 'function' && buf instanceof Buffer) {
    return new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength)
  }
  return buf
}

class Encoding {
  constructor (encoding_name) {
    if (typeof encoding_name !== 'string') {
      throw new Error('encoding_name must be a string')
    }
    this._raw = txError(() => new bindgen.Encoding(encoding_name))
  }

  encode (data) {
    data = checkFixBuffer(data)
    if (!(data instanceof Uint8Array)) {
      throw new Error('data must be a Uint8Array')
    }
    return txError(() => this._raw.encode(data))
  }

  decode (data) {
    if (typeof data !== 'string') {
      throw new Error('data must be a string')
    }
    return txError(() => this._raw.decode(data))
  }

  is_corrupt (data) {
    if (typeof data !== 'string') {
      throw new Error('data must be a string')
    }
    return txError(() => this._raw.is_corrupt(data))
  }
}

exports.Encoding = Encoding
