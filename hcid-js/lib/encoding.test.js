/* eslint no-new:0 */

const { Encoding } = require('..')
const { expect } = require('chai')

const TEST_HEX_1 =
    '0c71db50d35d760b0ea2002ff20147c7c3a8e8030d35ef28ed1adaec9e329aba'
const TEST_ID_1 =
    'HcKciDds5OiogymxbnHKEabQ8iavqs8dwdVaGdJW76Vp4gx47tQDfGW4OWc9w5i'

describe('Encoding Suite', () => {
  let enc = null

  beforeEach(() => {
    enc = new Encoding('hck0')
  })

  afterEach(() => {
    enc = null
  })

  it('should encode', () => {
    const buf = Buffer.from(TEST_HEX_1, 'hex')
    expect(enc.encode(buf))
      .equals(TEST_ID_1)
  })

  it('should decode', () => {
    expect(Buffer.from(enc.decode(TEST_ID_1)).toString('hex'))
      .equals(TEST_HEX_1)
  })

  it('should determine not is_corrupt', () => {
    expect(enc.is_corrupt(TEST_ID_1))
      .equals(false)
  })

  it('should determine is_corrupt', () => {
    expect(enc.is_corrupt(TEST_ID_1.substr(0, 10) + 'A' + TEST_ID_1.substr(11)))
      .equals(true)
  })
})
