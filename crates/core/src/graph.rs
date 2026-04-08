use std::collections::HashMap;

pub struct Graph<VId, E = (), V = ()> {
    vertices: HashMap<VId, V>,
    edges: HashMap<VId, Vec<(VId, E)>>,
}

impl<VId, E, V> Graph<VId, E, V> {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    // pub fn add_vertex(&mut self, vid: VId, vertex: V) {
    //     self.vertices.insert(vid, vertex);
    // }
    //
    // pub fn add_edge(&mut self, from: VId, to: VId, value: E) {
    //     self.edges.entry(from).or_default().push((to, value));
    // }

    // pub fn get_vertex(&self, id: &VId) -> Option<&V> {
    //     self.vertices.get(id)
    // }
    //
    // pub fn get_edges(&self, id: &VId) -> Option<&Vec<(VId, E)>> {
    //     self.edges.get(id)
    // }
}