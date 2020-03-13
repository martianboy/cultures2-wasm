//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

#[path = "../src/lib.rs"] mod lib;
#[path = "../src/pcx.rs"] mod pcx;

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use pcx::*;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_tessellate() {
    lib::triangulate(2, 2, &[0, 0, 0, 0]);
}

#[wasm_bindgen_test]
fn test_pcx() {
    match File::open("/mnt/c/Users/Abbas/Projects/Personal/cultures2-wasm/tests/tran_desertbrown.pcx") {
        Ok(file) => {
            let mut buf_reader = BufReader::new(file);
            let mut buffer = Vec::new();

            buf_reader.read_to_end(&mut buffer);
            let res = pcx_header(&buffer[..]);
            if let Ok((_i, header)) = res {
                assert_eq!(header.magic, 0x0A);
                assert_eq!(header.version, 0x05);
                assert_eq!(header.encoding_method, 0x01);
                assert_eq!(header.bits_per_pixel, 0x08);
            } else {
                panic!("Hey!");
            }
        },
        Err(e) => {
            panic!("Could not open the pcx file!");
        }
    }
}