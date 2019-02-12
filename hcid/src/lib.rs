extern crate data_encoding;
extern crate reed_solomon;

#[derive(Debug, PartialEq, Clone)]
pub struct HcidError(String);

impl std::fmt::Display for HcidError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HcidError {
    fn description(&self) -> &str {
        &self.0
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl From<data_encoding::SpecificationError> for HcidError {
    fn from(error: data_encoding::SpecificationError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<data_encoding::DecodeError> for HcidError {
    fn from(error: data_encoding::DecodeError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<reed_solomon::DecoderError> for HcidError {
    fn from(error: reed_solomon::DecoderError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<std::num::ParseIntError> for HcidError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self(format!("{:?}", error))
    }
}

pub type HcidResult<T> = Result<T, HcidError>;

pub struct HcidEncodingConfig {
    pub key_byte_count: usize,
    pub base_parity_byte_count: usize,
    pub cap_parity_byte_count: usize,
    pub prefix: Vec<u8>,
    pub prefix_cap: Vec<u8>,
    pub cap_segment_char_count: usize,
    pub encoded_char_count: usize,
}

pub struct HcidEncoding {
    b32: data_encoding::Encoding,
    config: HcidEncodingConfig,
    rs_enc: reed_solomon::Encoder,
    rs_dec: reed_solomon::Decoder,
}

impl HcidEncoding {
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

    pub fn decode(&self, data: &str) -> HcidResult<Vec<u8>> {
        let (data, erasures) = self.pre_decode(data)?;

        let data = self.rs_dec.correct(&data, Some(&erasures[..]))?;

        Ok(data[0..self.config.key_byte_count].to_vec())
    }

    pub fn is_corrupt(&self, data: &str) -> HcidResult<bool> {
        let (data, erasures) = self.pre_decode(data)?;

        if erasures.len() > 0 {
            return Ok(true);
        }

        Ok(self.rs_dec.is_corrupted(&data))
    }

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

fn cap_decode(
    char_offset: usize,
    byte_offset: usize,
    data: &[u8],
    char_erasures: &mut Vec<u8>,
    byte_erasures: &mut Vec<u8>,
) -> HcidResult<u8> {
    let mut bin = String::new();
    for i in 0..data.len() {
        if char_erasures[char_offset + i] == b'1' {
            // parity byte will be marked as an erasure
            bin.clear();
            break;
        }

        let c = data[i];

        // is alpha
        if c >= b'A' && c <= b'Z' {
            bin.push('1');
        } else if c >= b'a' && c <= b'z' {
            bin.push('0');
        }
        if bin.len() >= 8 {
            break;
        }
    }

    if bin.len() < 8 || &bin == "11111111" || &bin == "00000000" {
        byte_erasures[byte_offset] = b'1';
        return Ok(0);
    }

    Ok(u8::from_str_radix(&bin, 2)?)
}

fn b32_correct(data: &[u8], char_erasures: &mut Vec<u8>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();

    let len = data.len();
    for i in 0..len {
        out.push(match data[i] {
            b'0' => b'O',
            b'1' | b'L' => b'I',
            b'l' => b'i',
            b'2' => b'Z',
            b'A'..=b'Z' | b'a'..=b'z' | b'3'..=b'9' => data[i],
            _ => {
                char_erasures[i] = b'1';
                b'A'
            }
        })
    }

    out
}

fn char_lower(c: &mut u8) {
    if *c >= b'A' && *c <= b'Z' {
        *c = *c + 32;
    }
}

fn char_upper(c: &mut u8) {
    if *c >= b'a' && *c <= b'z' {
        *c = *c - 32;
    }
}

/// encode `bin` into `seg` as capitalization
/// if `min` is not met, lowercase the whole thing
/// as an indication that we did not have enough alpha characters
fn cap_encode_bin(seg: &mut [u8], bin: &[u8], min: usize) -> HcidResult<()> {
    let mut count = 0;
    let mut bin_idx = 0;
    for c in seg.iter_mut() {
        if bin_idx >= bin.len() {
            char_lower(c);
            continue;
        }
        // is alpha
        if (*c >= b'A' && *c <= b'Z') || (*c >= b'a' && *c <= b'z') {
            count += 1;
            // is 1
            if bin[bin_idx] == b'1' {
                char_upper(c);
            } else {
                char_lower(c);
            }
            bin_idx += 1;
        }
    }
    if count < min {
        for c in seg.iter_mut() {
            char_lower(c);
        }
    }
    Ok(())
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
