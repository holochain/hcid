use super::HcidResult;

/// pull parity bytes out that were encoded as capitalization
/// translate character-level erasures into byte-level erasures
pub fn cap_decode(
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

/// correct and transliteration faults
/// also note any invalid characters as erasures (character-level)
pub fn b32_correct(data: &[u8], char_erasures: &mut Vec<u8>) -> Vec<u8> {
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

/// modify a character to be ascii upper-case in-place
pub fn char_lower(c: &mut u8) {
    if *c >= b'A' && *c <= b'Z' {
        *c = *c + 32;
    }
}

/// modify a character to be ascii lower-case in-place
pub fn char_upper(c: &mut u8) {
    if *c >= b'a' && *c <= b'z' {
        *c = *c - 32;
    }
}

/// encode `bin` into `seg` as capitalization
/// if `min` is not met, lowercase the whole thing
/// as an indication that we did not have enough alpha characters
pub fn cap_encode_bin(seg: &mut [u8], bin: &[u8], min: usize) -> HcidResult<()> {
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
