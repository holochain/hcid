extern crate getopts;
extern crate hex;

use std::{
    env, process::exit
};
use getopts::Options;

use hcid;

mod error;

const DETAIL: &str = "

    Encodes or decodes (and corrects) 0 or more hex or Holochain encoded 256-bit
values; the default behavior is to --encode, meaning we will output the Holochain-
encoded result.

    If the value already appears to be encoded, it will first be decoded, and 
then (if found valid), re-encoded.  This provides a means to error-correct and
re-encode keys that may have been partially corrupted, or which are correct but
have lost their case-encoded Reed-Solomon parity data.  The --decode option (alone)
will result in output of the decoded hex values.
";

/// print_usage -- print usage message to stderr
fn print_usage(program: &str, opts: &Options) {
    let brief = opts.short_usage(program);
    eprintln!("{}\n{}", opts.usage(&brief), DETAIL);
}

/// fail -- print usage and error message to stderr, and exit with non-zero exit status
fn fail(message: &str, program: &str, opts: &Options) -> ! {
    eprintln!("{}\n", message);
    print_usage(program, opts);
    exit(1);
}

/// correct -- decode a hex- or Holochain-encoded value, return 256-bit and encoded String values
///
///     To support potential encodings that carry other than 256-bit values, we'll return a Vec<u8>
/// instead of an array.
pub fn correct(
    codec: &hcid::HcidEncoding,
    value: &str
) -> error::HcidResult<(Vec<u8>, String)> {
    // First attempt to harvest as a hex value, then as a Holochain-encoded value
    let dec_val: Vec<u8> = match hex::decode(&value) {
        Ok(v) => v,
        Err(_) => {
            // Wasn't a hex string; see if its a Holochain-encoded ID
            match codec.decode(&value) {
                Ok(v) => v,
                Err(e) => return Err(format!(
                    "Failed to decode hex or Hc-encoded ID: {}",
                    e).into()),
            }
        }
    };
    // OK, we have a Vec<u8>; see if its the right size to encode as the Hc-encoded ID
    let enc_val = match codec.encode(&dec_val) {
        Ok(r) => r,
        Err(e) => return Err(format!(
            "Failed to encode {}-bit data value into Hc-encoded ID: {}",
            dec_val.len() * 8, e).into()),
    };
    Ok((dec_val, enc_val))
}
    
/// main -- process argv options and emit converted values, terminating w/ non-zero status on error
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt( "t", "type",   "Specify a Holochain Hc-encoding (default: HcS0)", "TYPE");
    opts.optflag("e", "encode", "Encode 256-bit hex into Hc-encoded symbol (the default)");
    opts.optflag("d", "decode", "Decode Hc-encoded symbol into 256-bit hex");
    opts.optflag("h", "help",   "Print this help menu");
                
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            fail(&f.to_string(), &program, &opts);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    // Holochain key encoding types are all lower-case, but the prefixes supplied
    // are typically mapped to HcS... case; handle either.
    let hcid_encoding = matches.opt_str("t").unwrap_or("HcS0".to_string());

    // Process remaining non-option arguments as hex or hcid
    let hcid_codec = match hcid::HcidEncoding::with_kind(&hcid_encoding.to_lowercase()) {
        Ok(c) => c,
        Err(e) => fail(&format!(
            "Failed to instantiate Holochain {} codec: {}",
            &hcid_encoding, e
        ), &program, &opts),
    };
    matches.free
        .iter()
        .for_each(|v| {
            let (dec_val, enc_val) = match correct( &hcid_codec, v ) {
                Ok((d,e)) => (d,e),
                Err(e) => fail(&format!(
                            "Failed to extract {:?}-encoded ID from {}: {}",
                            &hcid_encoding, &v, e
                ), &program, &opts),
            };
            println!("{}", if matches.opt_present("e") || ! matches.opt_present("d") {
                enc_val
            } else {
                hex::encode(dec_val)
            })

            /*
            if matches.opt_present("d") {
                println!(
                    "{}",
                    match hcid_codec.decode(&v) {
                        Ok(r) => hex::encode(r),
                        Err(e) => fail(&format!(
                            "Failed to decode Holochain {} from {} into data value: {}",
                            &hcid_encoding, &v, e
                        ), &program, &opts),
                    }
                );
            } else { // if matches.opt_present("e") // the default
                println!(
                    "{}",
                    match hcid_codec.encode(
                        &match <[u8; 32] as FromHex>::from_hex(&v) {
                            Ok(d) => d,
                            Err(e) => fail(&format!(
                                "Failed to decode 256-bit data value from hex {:?}: {}",
                                &v, e
                            ), &program, &opts),
                        }
                    ) {
                        Ok(r) => r,
                        Err(e) => fail(&format!(
                            "Failed to encode 256-bit data value {} into Holochain {}: {}",
                            &v, &hcid_encoding, e
                        ), &program, &opts),
                    }
                );
            };
            */
        });

    (())
}
