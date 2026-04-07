wai_bindgen_rust::export!("graphit.wai");

pub struct Graphit;

impl graphit::Graphit for Graphit {
    fn version() -> String {
        "0.1.0".to_string()
    }
    
    fn add_vertex(vrtx: u32) -> u32 {
        vrtx * 33
    }
}