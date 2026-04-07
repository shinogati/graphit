use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn delete_vertex(vrtx: u32) -> String {
    format!("Deleting: {}", vrtx)
}
