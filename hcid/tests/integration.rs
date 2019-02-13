extern crate serde_json;
extern crate data_encoding;
extern crate hcid;

use data_encoding::HEXLOWER_PERMISSIVE as hex;
fn read_hex(h: &str) -> Vec<u8> {
    hex.decode(h.as_bytes()).unwrap()
}

static FIXTURES: &'static str = include_str!("../../test/fixtures.json");

fn test_correct(id: &str, data: &[u8]) {
    let e = hcid::with_hck0().unwrap();
    assert!(!e.is_corrupt(id).unwrap());
    let r = e.decode(id).unwrap();
    assert_eq!(data, r.as_slice());
    let r = e.encode(data).unwrap();
    assert_eq!(id, r);
}

fn test_correctable(id: &str, data: &[u8], correct_id: &str) {
    let e = hcid::with_hck0().unwrap();
    assert!(e.is_corrupt(id).unwrap());
    let r = e.decode(id).unwrap();
    assert_eq!(data, r.as_slice());
    let r = e.encode(&r).unwrap();
    assert_eq!(correct_id, r);
}

fn test_errant_id(id: &str, err: &str) {
    let e = hcid::with_hck0().unwrap();
    assert!(e.is_corrupt(id).unwrap());
    let r = e.decode(id).unwrap_err();
    assert_eq!(err, format!("{:?}", r));
}

fn test_errant_data(data: &[u8], err: &str) {
    let e = hcid::with_hck0().unwrap();
    let r = e.encode(data).unwrap_err();
    assert_eq!(err, format!("{:?}", r));
}

fn test_hck0(test: &serde_json::Value) {
    let test = test.as_object().unwrap();

    for t in test["correct"].as_array().unwrap().iter() {
        let id = String::from(t[0].as_str().unwrap());
        let data = read_hex(&String::from(t[1].as_str().unwrap()));
        test_correct(&id, &data);
    }

    for t in test["correctable"].as_array().unwrap().iter() {
        let id = String::from(t[0].as_str().unwrap());
        let data = read_hex(&String::from(t[1].as_str().unwrap()));
        let correct_id = String::from(t[2].as_str().unwrap());
        test_correctable(&id, &data, &correct_id);
    }

    for t in test["errantId"].as_array().unwrap().iter() {
        let id = String::from(t[0].as_str().unwrap());
        let err = String::from(t[1].as_str().unwrap());
        test_errant_id(&id, &err);
    }

    for t in test["errantData"].as_array().unwrap().iter() {
        let data = read_hex(&String::from(t[0].as_str().unwrap()));
        let err = String::from(t[1].as_str().unwrap());
        test_errant_data(&data, &err);
    }
}

#[test]
fn it_can_execute_fixtures() {
    let fixtures: serde_json::Value = serde_json::from_str(FIXTURES).unwrap();
    let fixtures = fixtures.as_object().unwrap();

    test_hck0(&fixtures["hck0"]);
}
