//! Utilities for encoding / decoding basic base32

/// 5 bit mask
const MASK: usize = 31;

/// holochain base32 alphabet
static ALPHABET: &'static [u8] = b"ABCDEFGHIJKMNOPQRSTUVWXYZ3456789";

/// reverse lookup table for alphabet positioning (ascii - 51)
static REV_LOOKUP: &'static [usize] = &[
    25, 26, 27, 28, 29, 30, 31,     // 0, 1, 2, 3, 4, 5, 6,
    0, 0, 0, 0, 0, 0, 0,            // 7, 8, 9, 10, 11, 12, 13,
    0, 1, 2, 3, 4, 5, 6,            // 14, 15, 16, 17, 18, 19, 20,
    7, 8, 9, 10,                    // 21, 22, 23, 24,
    0,                              // 25,
    11, 12, 13, 14, 15, 16, 17,     // 26, 27, 28, 29, 30, 31, 32,
    18, 19, 20, 21, 22, 23, 24,     // 33, 34, 35, 36, 37, 38, 39,
];

/// encode a byte buffer into basic holochain base32
pub fn encode (data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];

    let mut bits: usize = 0;
    let mut tmp: usize = 0;

    for c in data {
        tmp = (tmp << 8) | (0xff & c) as usize;
        bits += 8;

        while bits > 5 {
            bits -= 5;
            out.push(ALPHABET[MASK & (tmp >> bits)])
        }
    }

    if bits > 0 {
        out.push(ALPHABET[MASK & (tmp << (5 - bits))])
    }

    out
}

/// decode an already sanitized holochain base32 string into a byte buffer
pub fn decode (data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];

    let mut bits: usize = 0;
    let mut tmp: usize = 0;

    for c in data {
        let v = REV_LOOKUP[(c - 51) as usize];
        tmp = (tmp << 5) | v;
        bits += 5;

        if bits >= 8 {
            bits -= 8;
            out.push((0xff & (tmp >> bits)) as u8)
        }
    }

    if bits >= 5 || (0xff & (tmp << (8 - bits))) != 0 {
        panic!("unexpected eof");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_encode_1() {
        assert_eq!(b"AEBAGBASWMGQ9VRB".to_vec(), encode(
            &[0x01, 0x02, 0x03, 0x04, 0x11, 0xaa, 0xcc, 0xff, 0xd2, 0x01]));
    }

    #[test]
    fn it_should_encode_2() {
        assert_eq!(b"AEBAGBASWMGQ9VR".to_vec(), encode(
            &[0x01, 0x02, 0x03, 0x04, 0x11, 0xaa, 0xcc, 0xff, 0xd2]));
    }

    #[test]
    fn it_should_encode_3() {
        assert_eq!(b"AEBAGBASWMGA".to_vec(), encode(
            &[0x01, 0x02, 0x03, 0x04, 0x11, 0xaa, 0xcc]));
    }

    #[test]
    fn it_should_decode_1() {
        assert_eq!(
            &format!("{:?}", [0x01, 0x02, 0x03, 0x04, 0x11, 0xaa, 0xcc, 0xff, 0xd2, 0x01]),
            &format!("{:?}", &decode(b"AEBAGBASWMGQ9VRB")));
    }
}
