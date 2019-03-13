/* eslint camelcase:0 */

import * as bindgen from './bindgen'
import { booted } from './bindgen_bg'

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

export class Encoding {
  constructor (encoding_name) {
    return booted.then(() => {
      if (typeof encoding_name !== 'string') {
        throw new Error('encoding_name must be a string')
      }
      this._raw = txError(() => new bindgen.Encoding(encoding_name))
      return this
    })
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
