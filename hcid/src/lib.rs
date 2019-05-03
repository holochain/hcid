extern crate reed_solomon;

mod error;
mod b32;
pub use error::HcidError;

mod util;
use util::{b32_correct, cap_decode, cap_encode_bin, char_upper};

/// hcid Result type
pub type HcidResult<T> = Result<T, HcidError>;

/* XXX
 *
 * HcK v0 hex:     0x389424
 * HcK v1 hex:     0x389524
 * HcA v0 hex:     0x388024
 * HcA v1 hex:     0x388124
 * HcS v0 hex:     0x38a224
 * HcS v1 hex:     0x38a324
 *
 * XXX
 */

/// create a hck0 encoding instance
/// version zero of keys prefixed with `HcK`
pub fn with_hck0() -> HcidResult<HcidEncoding> {
    HcidEncoding::new(HcidEncodingConfig {
        key_byte_count: 32,
        base_parity_byte_count: 4,
        cap_parity_byte_count: 4,
        prefix: vec![0x38, 0x94, 0x24],
        prefix_cap: b"101".to_vec(),
        cap_segment_char_count: 15,
        encoded_char_count: 63,
    })
}

/// create a hca0 encoding instance
/// version zero of keys prefixed with `HcA`
pub fn with_hca0() -> HcidResult<HcidEncoding> {
    HcidEncoding::new(HcidEncodingConfig {
        key_byte_count: 32,
        base_parity_byte_count: 4,
        cap_parity_byte_count: 4,
        prefix: vec![0x38, 0x80, 0x24],
        prefix_cap: b"101".to_vec(),
        cap_segment_char_count: 15,
        encoded_char_count: 63,
    })
}

/// create a hcs0 encoding instance
/// version zero of keys prefixed with `HcS`
pub fn with_hcs0() -> HcidResult<HcidEncoding> {
    HcidEncoding::new(HcidEncodingConfig {
        key_byte_count: 32,
        base_parity_byte_count: 4,
        cap_parity_byte_count: 4,
        prefix: vec![0x38, 0xa2, 0x24],
        prefix_cap: b"101".to_vec(),
        cap_segment_char_count: 15,
        encoded_char_count: 63,
    })
}

/// represents an encoding configuration for hcid rendering and parsing
pub struct HcidEncodingConfig {
    /// byte count of actuall key data that will be encoded
    pub key_byte_count: usize,
    /// parity bytes that will be encoded directly into the base32 string (appended after key)
    pub base_parity_byte_count: usize,
    /// parity bytes that will be encoded in the alpha capitalization (appended after base parity)
    pub cap_parity_byte_count: usize,
    /// bytes to prefix before rendering to base32
    pub prefix: Vec<u8>,
    /// binary indication of the capitalization for prefix characters
    pub prefix_cap: Vec<u8>,
    /// how many characters are in a capitalization parity segment
    pub cap_segment_char_count: usize,
    /// how many characters long the fully rendered base32 string should be
    pub encoded_char_count: usize,
}

/// an instance that can encode / decode a particular hcid encoding configuration
pub struct HcidEncoding {
    config: HcidEncodingConfig,
    rs_enc: reed_solomon::Encoder,
    rs_dec: reed_solomon::Decoder,
}

impl HcidEncoding {
    /// create a new HcidEncoding instance from given HcidEncodingConfig
    pub fn new(config: HcidEncodingConfig) -> HcidResult<Self> {
        // set up a reed-solomon encoder with proper parity count
        let rs_enc = reed_solomon::Encoder::new(
            config.base_parity_byte_count + config.cap_parity_byte_count,
        );

        // set up a reed-solomon decoder with proper parity count
        let rs_dec = reed_solomon::Decoder::new(
            config.base_parity_byte_count + config.cap_parity_byte_count,
        );

        Ok(Self {
            //b32,
            config,
            rs_enc,
            rs_dec,
        })
    }

    /// encode a string to base32 with this instance's configuration
    pub fn encode(&self, data: &[u8]) -> HcidResult<String> {
        if data.len() != self.config.key_byte_count {
            return Err(HcidError(String::from(format!(
                "BadDataLen:{},Expected:{}",
                data.len(),
                self.config.key_byte_count
            ))));
        }

        // generate reed-solomon parity bytes
        let full_parity = self.rs_enc.encode(data);

        // extract the bytes that will be encoded as capitalization
        let cap_bytes = &full_parity[full_parity.len() - self.config.cap_parity_byte_count..];

        // base is the bytes that will be base32 encoded
        let mut base = self.config.prefix.clone();
        base.extend_from_slice(
            &full_parity[0..full_parity.len() - self.config.cap_parity_byte_count],
        );

        // do the base32 encoding
        //let mut base32 = self.b32.encode(&base).into_bytes();
        let mut base32 = b32::encode(&base);

        if base32.len() != self.config.encoded_char_count {
            return Err(HcidError(String::from(format!(
                "InternalGeneratedBadLen:{},Expected:{}",
                base32.len(),
                self.config.encoded_char_count
            ))));
        }

        // capitalize the prefix with a fixed scheme
        cap_encode_bin(
            &mut base32[0..self.config.prefix_cap.len()],
            &self.config.prefix_cap,
            3,
        )?;

        // iterate over segments, applying parity capitalization
        for i in 0..cap_bytes.len() {
            let seg_start = self.config.prefix_cap.len() + (i * self.config.cap_segment_char_count);
            let seg = &mut base32[seg_start..seg_start + self.config.cap_segment_char_count];
            let bin = format!("{:08b}", cap_bytes[i]).into_bytes();
            cap_encode_bin(seg, &bin, 8)?;
        }
        
        // we only use ascii characters
        // use unchecked for performance / so we don't allocate again
        unsafe {
            // return the result as a String for ease of use
            Ok(String::from_utf8_unchecked(base32))
        }
    }

    /// decode the data from a base32 string with this instance's configuration.  Reed-Solomon can
    /// correct up to 1/2 its parity size worth of erasures (if no other errors are present).
    pub fn decode(&self, data: &str) -> HcidResult<Vec<u8>> {
        // get our parsed data with erasures
        let (data, erasures) = self.pre_decode(data)?;

        if erasures.len() > ( self.config.base_parity_byte_count + self.config.cap_parity_byte_count ) / 2 {
            // our reed-solomon library makes bad corrections once erasure count exceeds 1/2 the
            // parity count (it takes 2 parity symbols to find/correct one error, 1 parity symbol to
            // correct a known erasure)
            return Err(HcidError(String::from("TooManyErrors")));
        }

        // optimise for the case where there are no transcription errors
        // this makes correcting more expensive if there *are*,
        // but on average makes the system more efficient
        if self.pre_is_corrupt(&data, &erasures)? {
            // apply reed-solomon correction
            // will "throw" on too many errors
            Ok(
                self.rs_dec.correct(&data, Some(&erasures[..]))?[0..self.config.key_byte_count]
                    .to_vec(),
            )
        } else {
            Ok(data[0..self.config.key_byte_count].to_vec())
        }
    }

    /// a lighter-weight check to determine if a base32 string is corrupt
    pub fn is_corrupt(&self, data: &str) -> HcidResult<bool> {
        // get our parsed data with erasures
        let (data, erasures) = match self.pre_decode(data) {
            Ok(v) => v,
            Err(_) => return Ok(true),
        };

        match self.pre_is_corrupt(&data, &erasures) {
            Ok(v) => Ok(v),
            Err(_) => Ok(true),
        }
    }

    /// internal helper for is_corrupt checking
    fn pre_is_corrupt(&self, data: &[u8], erasures: &[u8]) -> HcidResult<bool> {
        // if we have any erasures, we can exit early
        if erasures.len() > 0 {
            return Ok(true);
        }

        // slightly more efficient reed-solomon corruption check
        Ok(self.rs_dec.is_corrupted(&data))
    }

    /// internal helper for preparing decoding
    fn pre_decode(&self, data: &str) -> HcidResult<(Vec<u8>, Vec<u8>)> {
        if data.len() != self.config.encoded_char_count {
            return Err(HcidError(String::from(format!(
                "BadIdLen:{},Expected:{}",
                data.len(),
                self.config.encoded_char_count
            ))));
        }

        let key_base_byte_size = self.config.key_byte_count + self.config.base_parity_byte_count;
        // All char_erasures are indexed from the 0th char of the full codeword w/ prefix, but
        // byte_erasures are indexed from the 0th byte of the key+parity (ie. without the prefix).
        // Any byte of key, or base/cap parity could be erased.
        let mut byte_erasures = vec![b'0'; key_base_byte_size + self.config.cap_parity_byte_count];
        let mut char_erasures = vec![b'0'; data.len()];

        // correct any transliteration errors into our base32 alphabet
        // marking any unrecognizable characters as char-level erasures
        let mut data = b32_correct(data.as_bytes(), &mut char_erasures);

        // Pull out the parity data that was encoded as capitalization.  If its erasure,
        // determine the 
        let mut cap_bytes: Vec<u8> = Vec::new();
        let mut all_zro = true;
        let mut all_one = true;
        for i in 0..self.config.cap_parity_byte_count {
            // For cap. parity, indexing starts after pre-defined Base-32 prefix
            let char_idx = self.config.prefix_cap.len() + (i * self.config.cap_segment_char_count);
            match cap_decode(
                char_idx,
                &data[char_idx..char_idx + self.config.cap_segment_char_count],
                &char_erasures
            )? {
                None => {
                    byte_erasures[key_base_byte_size + i] = b'1';
                    cap_bytes.push(0)
                }
                Some(parity) => {
                    if all_zro && parity != 0x00_u8 {
                        all_zro = false
                    }
                    if all_one && parity != 0xFF_u8 {
                        all_one = false
                    }
                    cap_bytes.push(parity)
                }
            }
        }

        // If either all caps or all lower case (or erasure), assume the casing was lost (eg. QR
        // code, or dns segment); mark all cap-derived parity as erasures.  This allows validation
        // of codeword if all remaining parity is intact and key is correct; since no parity
        // capacity remains, no correction will be attempted.  There is only a low probability that
        // any remaining errors will be detected, in this case.  However, we're no *worse* off than
        // if we had no R-S parity at all.
        if all_zro || all_one {
            for i in 0..self.config.cap_parity_byte_count {
                byte_erasures[key_base_byte_size + i] = b'1';
            }
        }

        // we have the cap data, uppercase everything
        for c in data.iter_mut() {
            char_upper(c);
        }

        // do the base32 decode
        //let mut data = self.b32.decode(&data)?;
        let mut data = b32::decode(&data);

        if &data[0..self.config.prefix.len()] != self.config.prefix.as_slice() {
            return Err(HcidError(String::from("PrefixMismatch")));
        }

        // remove the prefix bytes
        data.drain(0..self.config.prefix.len());

        // append our cap parity bytes
        data.append(&mut cap_bytes);

        // Sort through the char-level erasures (5 bits), and associate them with byte-level data (8
        // bits) -- in the (now prefix-free) data buffer, so that we mark the proper erasures for
        // reed-solomon correction.  Some of these chars span multiple bytes... we need to mark both.
        for i in self.config.prefix_cap.len()..char_erasures.len() {
            let c = char_erasures[i];
            if c == b'1' {
                // 1st and last bit of 5-bit segment may index different bytes
                byte_erasures[( i * 5 + 0 ) / 8 - self.config.prefix.len()] = b'1';
                byte_erasures[( i * 5 + 4 ) / 8 - self.config.prefix.len()] = b'1';
            }
        }

        // translate erasures into the form expected by our reed-solomon lib
        let mut erasures: Vec<u8> = Vec::new();
        for i in 0..byte_erasures.len() {
            if byte_erasures[i] == b'1' {
                data[i] = 0;
                erasures.push(i as u8);
            }
        }

        Ok((data, erasures))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_HEX_1: &'static str =
        "0c71db50d35d760b0ea2002ff20147c7c3a8e8030d35ef28ed1adaec9e329aba";
    static TEST_ID_1: &'static str =
        "HcKciDds5OiogymxbnHKEabQ8iavqs8dwdVaGdJW76Vp4gx47tQDfGW4OWc9w5i";

    #[test]
    fn it_encodes_1() {
        let enc = with_hck0().unwrap();

        let input = hex::decode(TEST_HEX_1.as_bytes()).unwrap();
        let id = enc.encode(&input).unwrap();
        assert_eq!(TEST_ID_1, id);
    }

    #[test]
    fn it_decodes_1() {
        let enc = with_hck0().unwrap();

        let data = hex::encode(&enc.decode(TEST_ID_1).unwrap());
        assert_eq!(TEST_HEX_1, data);
    }
}
