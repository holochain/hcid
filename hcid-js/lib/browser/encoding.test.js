/* eslint no-new:0 */

const { Encoding } = require('../..')
const { expect } = require('chai')
const fixtures = require('../../../test/fixtures')

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
            it('' + i, async () => {
              const id = t[0]
              const data = t[1]
              expect(await enc.is_corrupt(id)).equals(false)
              expect(await enc.encode(Buffer.from(data, 'hex'))).equals(id)
              expect(Buffer.from(await enc.decode(id)).toString('hex')).equals(data)
            })
          }
        })

        describe('correctable', () => {
          for (let i = 0; i < f.correctable.length; ++i) {
            const t = f.correctable[i]
            it('' + i, async () => {
              const id = t[0]
              const data = t[1]
              const correctId = t[2]
              expect(await enc.is_corrupt(id)).equals(true)
              expect(await enc.encode(Buffer.from(data, 'hex'))).equals(correctId)
              expect(Buffer.from(await enc.decode(id)).toString('hex')).equals(data)
            })
          }
        })

        describe('errantId', () => {
          for (let i = 0; i < f.errantId.length; ++i) {
            const t = f.errantId[i]
            it('' + i, async () => {
              const id = t[0]
              const err = t[1]
              expect(await enc.is_corrupt(id)).equals(true)
              try {
                await enc.decode(id)
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
            it('' + i, async () => {
              const data = t[0]
              const err = t[1]
              try {
                await enc.encode(Buffer.from(data, 'hex'))
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

  describe('basic', () => {
    let enc = null

    beforeEach(async () => {
      enc = await new Encoding('hck0')
    })

    afterEach(() => {
      enc = null
    })

    it('should encode', async () => {
      const buf = Buffer.from(TEST_HEX_1, 'hex')
      expect(await enc.encode(buf))
        .equals(TEST_ID_1)
    })

    it('should decode', async () => {
      expect(Buffer.from(await enc.decode(TEST_ID_1)).toString('hex'))
        .equals(TEST_HEX_1)
    })

    it('should determine not is_corrupt', async () => {
      expect(await enc.is_corrupt(TEST_ID_1))
        .equals(false)
    })

    it('should determine is_corrupt', async () => {
      expect(await enc.is_corrupt(TEST_ID_1.substr(0, 10) + 'A' + TEST_ID_1.substr(11)))
        .equals(true)
    })
  })
})
