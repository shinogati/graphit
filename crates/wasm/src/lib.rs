mod helper;
use std::fmt;
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

impl fmt::Display for WasmEdgeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = if self.bidirectional { '◻' } else { '◼' };
        write!(f, "{}", symbol)?;
        Ok(())
    }
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

    /// Creates a cursor starting at the root vertex.
    /// Returns `undefined` if the graph has no root yet.
    #[wasm_bindgen(js_name = cursor)]
    pub fn cursor(&self) -> Option<WasmCursor> {
        self.0.root_vid().map(|vid| WasmCursor { current_vid: vid })
    }
}

/// A cursor that tracks a position inside a `WasmGraph`.
///
/// Because `wasm-bindgen` cannot express lifetime parameters on exported types,
/// the cursor owns only the current VID and receives a `&WasmGraph` on each
/// operation — mirroring the core `Cursor` interface.
#[wasm_bindgen]
pub struct WasmCursor {
    current_vid: u32,
}

#[wasm_bindgen]
impl WasmCursor {
    /// Creates a cursor positioned at the root vertex of `graph`.
    /// Mirrors `Cursor::new(&graph)` from core.
    /// Throws if the graph has no root vertex.
    #[wasm_bindgen(constructor)]
    pub fn new(graph: &WasmGraph) -> Result<WasmCursor, JsValue> {
        graph
            .0
            .root_vid()
            .map(|vid| WasmCursor { current_vid: vid })
            .ok_or_else(|| JsValue::from_str("Graph has no root vertex"))
    }

    /// The VID the cursor is currently pointing at.
    #[wasm_bindgen(getter, js_name = currentVid)]
    pub fn current_vid(&self) -> u32 {
        self.current_vid
    }

    /// Returns vertex data for the current position.
    #[wasm_bindgen(js_name = getNode)]
    pub fn get_node(&self, graph: &WasmGraph) -> Option<WasmVertex> {
        graph.0.get_vertex(self.current_vid).map(|v| WasmVertex {
            label: v.get_label().to_string(),
            step: v.get_step(),
        })
    }

    /// Returns the adjacency list for the current vertex.
    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&self, graph: &WasmGraph) -> Option<Vec<JsValue>> {
        graph.0.get_edges(self.current_vid).map(|edges| {
            edges
                .iter()
                .map(|(target_vid, edge)| {
                    JsValue::from(WasmEdgeEntry {
                        target_vid: *target_vid,
                        bidirectional: matches!(edge, Edge::Bidirectional(_)),
                    })
                })
                .collect()
        })
    }

    /// Moves the cursor to `vid` if it is a direct neighbour of the current vertex.
    /// Returns the new VID on success, or `undefined` if the move is not allowed.
    #[wasm_bindgen(js_name = moveTo)]
    pub fn move_to(&mut self, graph: &WasmGraph, vid: u32) -> Option<u32> {
        let reachable = graph
            .0
            .get_edges(self.current_vid)
            .map(|edges| edges.iter().any(|(t, _)| *t == vid))
            .unwrap_or(false);
        if reachable {
            self.current_vid = vid;
            Some(vid)
        } else {
            None
        }
    }
}

/// Creates a new graph with a root vertex labelled `name`.
#[wasm_bindgen(js_name = createGraph)]
pub fn create_graph(name: &str) -> WasmGraph {
    WasmGraph(Graph::new(name))
}