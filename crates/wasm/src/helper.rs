use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[cfg(not(target_family = "wasm"))]

use wasm_bindgen::prelude::*;

#[cfg(not(target_family = "wasm"))]
fn get_rand_vid(min: u32, max: u32) -> u32 {
    let num: u32 = rand::random_range(min..max);
    num
}

#[cfg(target_family = "wasm")]
pub fn get_rand_vid(min: usize, max: usize) -> u32 {
    ((random() * (max - min) as f64).floor() as usize + min) as u32
}