mod helper;

use wasm_bindgen::prelude::*;
use graphit_core::graph::{Edge, Graph, Vertex};

#[wasm_bindgen]
pub struct WasmGraph(Graph<Vertex, Edge>);

/// Vertex data returned from `WasmGraph::get_vertex`.
#[wasm_bindgen]
pub struct WasmVertex {
    label: String,
    step: u32,
}

#[wasm_bindgen]
impl WasmVertex {
    #[wasm_bindgen(getter)]
    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn step(&self) -> u32 {
        self.step
    }
}

/// A single entry in an adjacency list returned from `WasmGraph::get_edges`.
#[wasm_bindgen]
pub struct WasmEdgeEntry {
    target_vid: u32,
    bidirectional: bool,
}

#[wasm_bindgen]
impl WasmEdgeEntry {
    #[wasm_bindgen(getter, js_name = targetVid)]
    pub fn target_vid(&self) -> u32 {
        self.target_vid
    }

    /// `true` if the edge is `Bidirectional`, `false` if `Unidirectional`.
    #[wasm_bindgen(getter)]
    pub fn bidirectional(&self) -> bool {
        self.bidirectional
    }
}

#[wasm_bindgen]
impl WasmGraph {
    /// Returns the VID of the root vertex, or `undefined` if the graph is empty.
    #[wasm_bindgen(getter, js_name = rootVid)]
    pub fn root_vid(&self) -> Option<u32> {
        self.0.root_vid()
    }

    /// Adds a vertex with a generated VID and the given `label`.
    /// Returns the root VID (mirrors the core behaviour of `add_vertex`).
    #[wasm_bindgen(js_name = addVertex)]
    pub fn add_vertex(&mut self, vid: u32, label: &str) -> u32 {
        self.0.add_vertex(&vid, Vertex::new(label, None, None))
    }

    /// Adds a child vertex under `parent_vid`.
    /// `bidirectional` controls the edge direction.
    /// Returns the new vertex's VID, or `undefined` if `parent_vid` was not found.
    #[wasm_bindgen(js_name = addChild)]
    pub fn add_child(&mut self, parent_vid: u32, label: &str, bidirectional: bool) -> Option<u32> {
        let edge = if bidirectional {
            Some(Edge::Bidirectional(None))
        } else {
            None
        };
        self.0.add_child(&parent_vid, Vertex::new(label, None, None), edge)
    }

    /// Adds a bare edge between two existing vertices.
    #[wasm_bindgen(js_name = addEdge)]
    pub fn add_edge(&mut self, from: u32, to: u32, bidirectional: bool) {
        let edge = if bidirectional {
            Edge::Bidirectional(None)
        } else {
            Edge::Unidirectional(None)
        };
        self.0.add_edge(from, to, edge);
    }

    /// Returns the vertex for `vid`, or `undefined` if not found.
    #[wasm_bindgen(js_name = getVertex)]
    pub fn get_vertex(&self, vid: u32) -> Option<WasmVertex> {
        self.0.get_vertex(vid).map(|v| WasmVertex {
            label: v.get_label().to_string(),
            step: v.get_step(),
        })
    }

    /// Returns the adjacency list for `vid` as an array of `WasmEdgeEntry`,
    /// or `undefined` if `vid` has no edges.
    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&self, vid: u32) -> Option<Vec<JsValue>> {
        self.0.get_edges(vid).map(|edges| {
            edges
                .iter()
                .map(|(target_vid, edge)| {
                    let entry = WasmEdgeEntry {
                        target_vid: *target_vid,
                        bidirectional: matches!(edge, Edge::Bidirectional(_)),
                    };
                    JsValue::from(entry)
                })
                .collect()
        })
    }
}

/// Creates a new graph with a root vertex labelled `name`.
#[wasm_bindgen(js_name = createGraph)]
pub fn create_graph(name: &str) -> WasmGraph {
    WasmGraph(Graph::new(name))
}