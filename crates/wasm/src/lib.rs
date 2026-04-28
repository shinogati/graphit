mod helper;
use std::option::Option;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use graphit_core::graph::{Cursor, Edge, Graph, Vertex};

type InnerGraph = Graph<Vertex<String>, Edge<String>>;

#[wasm_bindgen]
pub struct WasmGraph(Rc<RefCell<InnerGraph>>);

/// Vertex data returned from `WasmGraph::get_vertex`.
#[wasm_bindgen]
pub struct WasmVertex {
    label: String,
    step: isize,
    payload: Option<String>,
}

#[wasm_bindgen]
impl WasmVertex {
    #[wasm_bindgen(getter)]
    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn step(&self) -> isize {
        self.step
    }

    /// The JSON payload stored on this vertex, or `undefined` if none has been set.
    #[wasm_bindgen(getter)]
    pub fn payload(&self) -> Option<String> {
        self.payload.clone()
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
    pub fn root_vid(&self) -> u32 {
        self.0.borrow().root_vid()
    }

    /// Adds a vertex with a generated VID and the given `label`.
    /// Adds a vertex with the given `vid` and `label`. Returns `vid`.
    #[wasm_bindgen(js_name = addVertex)]
    pub fn add_vertex(&mut self, vid: u32, label: &str) -> u32 {
        self.0.borrow_mut().add_vertex(&vid, Vertex::new(label, None, None))
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
        self.0.borrow_mut().add_child(&parent_vid, Vertex::new(label, None, None), edge)
    }

    /// Adds a bare edge between two existing vertices.
    #[wasm_bindgen(js_name = addEdge)]
    pub fn add_edge(&mut self, from: u32, to: u32, bidirectional: bool) {
        let edge = if bidirectional {
            Edge::Bidirectional(None)
        } else {
            Edge::Unidirectional(None)
        };
        self.0.borrow_mut().add_edge(from, to, edge);
    }

    /// Returns the vertex for `vid`, or `undefined` if not found.
    #[wasm_bindgen(js_name = getVertex)]
    pub fn get_vertex(&self, vid: u32) -> Option<WasmVertex> {
        self.0.borrow().get_vertex(vid).map(|v| WasmVertex {
            label: v.get_label().to_string(),
            step: v.get_step(),
            payload: v.get_payload().cloned(),
        })
    }

    /// Returns the adjacency list for `vid` as an array of `WasmEdgeEntry`,
    /// or `undefined` if `vid` has no edges.
    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&self, vid: u32) -> Option<Vec<JsValue>> {
        self.0.borrow().get_edges(vid).map(|edges| {
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

    /// Stores a JSON string as the payload of vertex `vid`.
    /// Returns `true` on success, `false` if `vid` does not exist.
    #[wasm_bindgen(js_name = setPayload)]
    pub fn set_payload(&mut self, vid: u32, data_str: String) -> bool {
        self.0.borrow_mut().set_vertex_payload(vid, data_str)
    }

    /// Stores a JSON string as the payload of the edge from `from` to `to`.
    /// Returns `true` on success, `false` if the edge does not exist.
    #[wasm_bindgen(js_name = setEdgePayload)]
    pub fn set_edge_payload(&mut self, from: u32, to: u32, data_str: String) -> bool {
        self.0.borrow_mut().set_edge_payload(from, to, data_str)
    }

    /// Returns the payload of the edge from `from` to `to`, or `undefined` if not set.
    #[wasm_bindgen(js_name = getEdgePayload)]
    pub fn get_edge_payload(&self, from: u32, to: u32) -> Option<String> {
        self.0.borrow().get_edges(from)?.iter()
            .find(|(t, _)| *t == to)
            .and_then(|(_, edge)| edge.get_payload().cloned())
    }

    /// Returns the children of the root vertex as an array of `WasmVertex`,
    /// or `undefined` if the root has no outgoing edges.
    #[wasm_bindgen(js_name = getChildren)]
    pub fn get_children(&self) -> Option<Vec<JsValue>> {
        self.0.borrow().get_children().map(|children| {
            children
                .into_iter()
                .map(|v| JsValue::from(WasmVertex {
                    label: v.get_label().to_string(),
                    step: v.get_step(),
                    payload: v.get_payload().cloned(),
                }))
                .collect()
        })
    }

    /// Returns the JSON payload of vertex `vid`, or `undefined` if none is set.
    #[wasm_bindgen(js_name = getPayload)]
    pub fn get_payload(&self, vid: u32) -> Option<String> {
        self.0.borrow().get_vertex(vid)?.get_payload().cloned()
    }

    /// Creates a cursor starting at the root vertex.
    #[wasm_bindgen(js_name = cursor)]
    pub fn cursor(&self) -> WasmCursor {
        WasmCursor(Cursor::new(Rc::clone(&self.0)))
    }
}

/// A cursor that tracks a position inside a `WasmGraph`.
/// Wraps the core `Cursor<String, String>` — all logic lives there.
#[wasm_bindgen]
pub struct WasmCursor(Cursor<String, String>);

#[wasm_bindgen]
impl WasmCursor {
    /// Creates a cursor positioned at the root vertex of `graph`.
    #[wasm_bindgen(constructor)]
    pub fn new(graph: &WasmGraph) -> WasmCursor {
        WasmCursor(Cursor::new(Rc::clone(&graph.0)))
    }

    /// The VID of the root vertex of the graph this cursor was created from.
    #[wasm_bindgen(getter, js_name = rootVid)]
    pub fn get_root(&self) -> u32 {
        self.0.get_root()
    }

    /// The VID the cursor is currently pointing at.
    #[wasm_bindgen(getter, js_name = currentVid)]
    pub fn current_vid(&self) -> u32 {
        self.0.get_current_node()
    }

    /// Returns vertex data for the current position.
    #[wasm_bindgen(js_name = getNode)]
    pub fn get_node(&self) -> Option<WasmVertex> {
        self.0.get_node().map(|v| WasmVertex {
            label: v.get_label().to_string(),
            step: v.get_step(),
            payload: v.get_payload().cloned(),
        })
    }

    /// Returns the adjacency list for the current vertex.
    #[wasm_bindgen(js_name = getEdges)]
    pub fn get_edges(&self) -> Option<Vec<JsValue>> {
        self.0.get_edges().map(|edges| {
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
    pub fn move_to(&mut self, vid: u32) -> Option<u32> {
        self.0.move_to(vid)
    }

    /// Steps back to the previous vertex in the traversal history.
    /// Returns the previous VID, or `undefined` if already at the root.
    #[wasm_bindgen(js_name = back)]
    pub fn back(&mut self) -> Option<u32> {
        self.0.back()
    }

    /// Returns the full traversal history as an array of VIDs, root first.
    #[wasm_bindgen(js_name = getPath)]
    pub fn get_path(&self) -> Vec<u32> {
        self.0.get_path()
    }

    /// Returns the cache entry for `key` as a `Uint8Array`, or `undefined` if not set.
    #[wasm_bindgen(js_name = getCacheItem)]
    pub fn get_cache_item(&self, key: u32) -> Option<Vec<u8>> {
        self.0.get_cache_item(key).cloned()
    }

    /// Removes and returns the cache entry for `key`, or `undefined` if not set.
    #[wasm_bindgen(js_name = removeCacheItem)]
    pub fn remove_cache_item(&mut self, key: u32) -> Option<Vec<u8>> {
        self.0.remove_cache_item(key)
    }

    /// Returns `true` if a cache entry exists for `key`.
    #[wasm_bindgen(js_name = containsCacheItem)]
    pub fn contains_cache_item(&self, key: u32) -> bool {
        self.0.contains_cache_item(key)
    }

    /// Stores `value` in the cache under `key`.
    #[wasm_bindgen(js_name = setCacheItem)]
    pub fn set_cache_item(&mut self, key: u32, value: Vec<u8>) {
        self.0.set_cache_item(key, value);
    }

    /// Removes all entries from the cache.
    #[wasm_bindgen(js_name = clearCache)]
    pub fn clear_cache(&mut self) {
        self.0.clear_cache();
    }

    /// Returns the graph this cursor was created from.
    #[wasm_bindgen(js_name = getGraph)]
    pub fn get_graph(&self) -> WasmGraph {
        WasmGraph(self.0.get_graph())
    }
}

/// Creates a new graph with a root vertex labelled `name`.
#[wasm_bindgen(js_name = createGraph)]
pub fn create_graph(name: &str) -> WasmGraph {
    WasmGraph(Rc::new(RefCell::new(InnerGraph::new(name))))
}
