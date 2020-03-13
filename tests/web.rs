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
