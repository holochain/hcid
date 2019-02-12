extern crate data_encoding;
extern crate reed_solomon;

mod error;
pub use error::HcidError;

mod util;
use util::{
    cap_decode,
    b32_correct,
    char_upper,
    cap_encode_bin,
};

/// hcid Result type
pub type HcidResult<T> = Result<T, HcidError>;

/// represents an encoding configuration for hcid rendering and parsing
pub struct HcidEncodingConfig {
    /// byte count of actuall key data that will be encoded
    pub key_byte_count: usize,
    /// parity bytes that will be encoded directly into the base32 string
    pub base_parity_byte_count: usize,
    /// parity bytes that will be encoded in the alpha capitalization
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
    b32: data_encoding::Encoding,
    config: HcidEncodingConfig,
    rs_enc: reed_solomon::Encoder,
    rs_dec: reed_solomon::Decoder,
}

impl HcidEncoding {
    /// create a new HcidEncoding instance from given HcidEncodingConfig
    pub fn new(config: HcidEncodingConfig) -> HcidResult<Self> {
        let mut spec = data_encoding::Specification::new();
        spec.symbols.push_str("ABCDEFGHIJKMNOPQRSTUVWXYZ3456789");
        let b32 = spec.encoding()?;

        let rs_enc = reed_solomon::Encoder::new(
            config.base_parity_byte_count + config.cap_parity_byte_count,
        );

        let rs_dec = reed_solomon::Decoder::new(
            config.base_parity_byte_count + config.cap_parity_byte_count,
        );

        Ok(Self {
            b32,
            config,
            rs_enc,
            rs_dec,
        })
    }

    /// create a hck0 encoding instance
    pub fn with_hck0() -> HcidResult<HcidEncoding> {
        Self::new(HcidEncodingConfig {
            key_byte_count: 32,
            base_parity_byte_count: 4,
            cap_parity_byte_count: 4,
            prefix: vec![0x38, 0x94, 0x24],
            prefix_cap: b"101".to_vec(),
            cap_segment_char_count: 15,
            encoded_char_count: 63,
        })
    }

    /// encode a string to base32 with this instance's configuration
    pub fn encode(&self, data: &[u8]) -> HcidResult<String> {
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
        let mut base32 = self.b32.encode(&base).into_bytes();

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

    /// decode the data from a base32 string with this instance's configuration
    pub fn decode(&self, data: &str) -> HcidResult<Vec<u8>> {
        let (data, erasures) = self.pre_decode(data)?;

        let data = self.rs_dec.correct(&data, Some(&erasures[..]))?;

        Ok(data[0..self.config.key_byte_count].to_vec())
    }

    /// a lighter-weight check to determine if a base32 string is corrupt
    pub fn is_corrupt(&self, data: &str) -> HcidResult<bool> {
        let (data, erasures) = self.pre_decode(data)?;

        if erasures.len() > 0 {
            return Ok(true);
        }

        Ok(self.rs_dec.is_corrupted(&data))
    }

    /// internal helper for preparing decoding
    fn pre_decode(&self, data: &str) -> HcidResult<(Vec<u8>, Vec<u8>)> {
        let key_byte_size = self.config.key_byte_count + self.config.base_parity_byte_count;
        let mut byte_erasures = vec![b'0'; key_byte_size];
        let mut char_erasures = vec![b'0'; data.len()];

        let mut data = b32_correct(data.as_bytes(), &mut char_erasures);

        let mut cap_bytes: Vec<u8> = Vec::new();
        for i in 0..self.config.cap_parity_byte_count {
            let char_idx = self.config.prefix_cap.len() + (i * self.config.cap_segment_char_count);
            cap_bytes.push(cap_decode(
                char_idx,
                key_byte_size + i,
                &data[char_idx..char_idx + self.config.cap_segment_char_count],
                &mut char_erasures,
                &mut byte_erasures,
            )?);
        }

        for c in data.iter_mut() {
            char_upper(c);
        }

        let mut data = self.b32.decode(&data)?;
        for _i in 0..self.config.prefix_cap.len() {
            data.remove(0);
        }
        data.append(&mut cap_bytes);

        for i in self.config.prefix_cap.len()..char_erasures.len() {
            let c = char_erasures[i];
            let byte_idx = (i as f64 - self.config.prefix_cap.len() as f64) * 5.0 / 8.0;
            let floor = byte_idx.floor() as usize;
            if c == b'1' {
                byte_erasures[floor] = b'1';
            }
            let ceil = byte_idx.ceil() as usize;
            if ceil >= self.config.key_byte_count + self.config.base_parity_byte_count {
                break;
            }
            if c == b'1' {
                byte_erasures[ceil] = b'1';
            }
        }

        let mut erasures: Vec<u8> = Vec::new();
        for i in 0..byte_erasures.len() {
            if byte_erasures[i] == b'1' {
                erasures.push(i as u8);
            }
        }

        Ok((data, erasures))
    }
}

#[cfg(test)]
mod tests {
    use data_encoding::HEXLOWER_PERMISSIVE as hex;

    use super::*;

    static TEST_HEX_1: &'static str =
        "0c71db50d35d760b0ea2002ff20147c7c3a8e8030d35ef28ed1adaec9e329aba";
    static TEST_ID_1: &'static str =
        "HcKciDds5OiogymxbnHKEabQ8iavqs8dwdVaGdJW76Vp4gx47tQDfGW4OWc9w5i";

    #[test]
    fn it_encodes_1() {
        let enc = HcidEncoding::with_hck0().unwrap();

        let input = hex.decode(TEST_HEX_1.as_bytes()).unwrap();
        let id = enc.encode(&input).unwrap();
        assert_eq!(TEST_ID_1, id);
    }

    #[test]
    fn it_decodes_1() {
        let enc = HcidEncoding::with_hck0().unwrap();

        let data = hex.encode(&enc.decode(TEST_ID_1).unwrap());
        assert_eq!(TEST_HEX_1, data);
    }
}
