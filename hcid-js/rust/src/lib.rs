extern crate hcid;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

pub type JsResult<T> = Result<T, JsValue>;
macro_rules! jserr {
    ($code:expr) => {
        match $code {
            Ok(v) => Ok(v),
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    };
}

#[wasm_bindgen]
pub struct Encoding(hcid::HcidEncoding);

#[wasm_bindgen]
impl Encoding {
    #[wasm_bindgen(constructor)]
    pub fn new(encoding_name: &str) -> JsResult<Encoding> {
        Ok(Encoding(jserr!(hcid::HcidEncoding::with_kind(encoding_name))?))
    }

    pub fn encode(&self, data: &[u8]) -> JsResult<String> {
        jserr!(self.0.encode(data))
    }

    pub fn decode(&self, data: &str) -> JsResult<Vec<u8>> {
        jserr!(self.0.decode(data))
    }

    pub fn is_corrupt(&self, data: &str) -> JsResult<bool> {
        jserr!(self.0.is_corrupt(data))
    }
}
