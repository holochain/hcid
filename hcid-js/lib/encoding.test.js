/* eslint no-new:0 */

const { Encoding } = require('..')
const { expect } = require('chai')
const fixtures = require('../../test/fixtures')

const TEST_HEX_1 =
    '0c71db50d35d760b0ea2002ff20147c7c3a8e8030d35ef28ed1adaec9e329aba'
const TEST_ID_1 =
    'HcKciDds5OiogymxbnHKEabQ8iavqs8dwdVaGdJW76Vp4gx47tQDfGW4OWc9w5i'

describe('Encoding Suite', () => {
  describe('fixtures', () => {
    for (let type in fixtures) {
      const f = fixtures[type]
      describe(type, () => {
        let enc = null

        beforeEach(async () => {
          enc = await new Encoding(type)
        })

        afterEach(() => {
          enc = null
        })

        describe('correct', () => {
          for (let i = 0; i < f.correct.length; ++i) {
            const t = f.correct[i]
            it('' + i, () => {
              const id = t[0]
              const data = t[1]
              expect(enc.is_corrupt(id)).equals(false)
              expect(enc.encode(Buffer.from(data, 'hex'))).equals(id)
              expect(Buffer.from(enc.decode(id)).toString('hex')).equals(data)
            })
          }
        })

        describe('correctable', () => {
          for (let i = 0; i < f.correctable.length; ++i) {
            const t = f.correctable[i]
            it('' + i, () => {
              const id = t[0]
              const data = t[1]
              const correctId = t[2]
              expect(enc.is_corrupt(id)).equals(true)
              expect(enc.encode(Buffer.from(data, 'hex'))).equals(correctId)
              expect(Buffer.from(enc.decode(id)).toString('hex')).equals(data)
            })
          }
        })

        describe('errantId', () => {
          for (let i = 0; i < f.errantId.length; ++i) {
            const t = f.errantId[i]
            it('' + i, () => {
              const id = t[0]
              const err = t[1]
              expect(enc.is_corrupt(id)).equals(true)
              try {
                enc.decode(id)
              } catch (e) {
                expect(e.toString()).equals('Error: ' + err)
                return
              }
              throw new Error('expected exception, got success')
            })
          }
        })

        describe('errantData', () => {
          for (let i = 0; i < f.errantData.length; ++i) {
            const t = f.errantData[i]
            it('' + i, () => {
              const data = t[0]
              const err = t[1]
              try {
                enc.encode(Buffer.from(data, 'hex'))
              } catch (e) {
                expect(e.toString()).equals('Error: ' + err)
                return
              }
              throw new Error('expected exception, got success')
            })
          }
        })
      })
    }
  })

  it('should error on bad kind', async () => {
    try {
      await new Encoding('bad')
    } catch (e) {
      expect(e.toString()).contains('invalid kind')
      return
    }
    throw new Error('expected exception, got success')
  })

  describe('basic', () => {
    let enc = null

    beforeEach(async () => {
      enc = await new Encoding('hck0')
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
})
