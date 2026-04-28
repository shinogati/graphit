use crate::helper::get_rand_vid;
use fnv::FnvHashMap;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Vertex<Payload = ()> {
    label: String,
    step: isize,
    payload: Option<Payload>,
    cache: std::collections::HashMap<u32, Vec<u8>>,
}
impl<Payload> Vertex<Payload> {
    pub fn new(name: &str, payload: Option<Payload>, p_step: Option<isize>) -> Self {
        let step = p_step.unwrap_or(-1);
        Self {
            label: name.to_string(),
            step: step + 1,
            payload,
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_step(&self) -> isize {
        self.step
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn get_payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    pub fn set_payload(&mut self, payload: Payload) {
        self.payload = Some(payload);
    }

    pub fn get_cache(&self) -> &std::collections::HashMap<u32, Vec<u8>> {
        &self.cache
    }

    pub fn get_cache_item(&self, key: u32) -> Option<&Vec<u8>> {
        self.cache.get(&key)
    }
    pub fn get_cache_item_mut(&mut self, key: u32) -> Option<&mut Vec<u8>> {
        self.cache.get_mut(&key)
    }
    pub fn remove_cache_item(&mut self, key: u32) -> Option<Vec<u8>> {
        self.cache.remove(&key)
    }
    pub fn contains_cache_item(&self, key: u32) -> bool {
        self.cache.contains_key(&key)
    }
    pub fn set_cache_item(&mut self, key: u32, value: Vec<u8>) {
        self.cache.insert(key, value);
    }
    pub fn get_cache_mut(&mut self) -> &mut std::collections::HashMap<u32, Vec<u8>> {
        &mut self.cache
    }
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}


#[derive(Debug, Clone)]
pub enum Edge<Payload = ()> {
    Unidirectional(Option<Payload>),
    Bidirectional(Option<Payload>),
}

impl<Payload> Edge<Payload> {
    pub fn get_payload(&self) -> Option<&Payload> {
        match self {
            Edge::Unidirectional(p) | Edge::Bidirectional(p) => p.as_ref(),
        }
    }

    pub fn set_payload(&mut self, payload: Payload) {
        match self {
            Edge::Unidirectional(p) | Edge::Bidirectional(p) => *p = Some(payload),
        }
    }
}

#[derive(Debug)]
pub struct Graph<Vertex, Edge> {
    root_vid: u32,
    vertices: FnvHashMap<u32, Vertex>,
    edges: FnvHashMap<u32, Vec<(u32, Edge)>>,
}

impl<VP, EP> Graph<Vertex<VP>, Edge<EP>>
{
    pub fn new(root_label: &str) -> Self {
        let vid = get_rand_vid(2, u32::MAX);
        let mut vertices = FnvHashMap::default();
        vertices.insert(vid, Vertex::<VP>::new(root_label, None, None));
        Self {
            root_vid: vid,
            vertices,
            edges: FnvHashMap::default(),
        }
    }

    /// Adds a vertex to the graph. Returns the VID of the inserted vertex.
    pub fn add_vertex(&mut self, vid: &u32, vertex: Vertex<VP>) -> u32 {
        self.vertices.insert(*vid, vertex);
        *vid
    }

    pub fn add_child(
        &mut self,
        parent_vid: &u32,
        mut child: Vertex<VP>,
        edge_type: Option<Edge<EP>>,
    ) -> Option<u32> {
        let rand_vid = get_rand_vid(2, u32::MAX);

        if self.vertices.contains_key(&parent_vid) {
            let p_step = self.vertices.get(&parent_vid).unwrap().step;
            child.step += p_step;
            self.vertices.insert(rand_vid, child);

            match edge_type {
                Some(e) => {
                    self.edges
                        .entry(*parent_vid)
                        .or_default()
                        .push((rand_vid, e));
                    return Some(rand_vid);
                }
                None => {
                    self.edges
                        .entry(*parent_vid)
                        .or_default()
                        .push((rand_vid, Edge::Unidirectional(None)));
                    return Some(rand_vid);
                }
            }
        }
        None
    }

    pub fn add_edge(&mut self, from: u32, to: u32, value: Edge<EP>) {
        self.edges.entry(from).or_default().push((to, value));
    }

    pub fn get_vertex(&self, vid: u32) -> Option<&Vertex<VP>> {
        self.vertices.get(&vid)
    }

    pub fn get_edges(&self, vid: u32) -> Option<&Vec<(u32, Edge<EP>)>> {
        self.edges.get(&vid)
    }

    pub fn get_children(&self) -> Option<Vec<Vertex<VP>>> where VP: Clone {
        self.edges.get(&self.root_vid).map(|edges| {
            edges
                .iter()
                .filter_map(|(target_vid, edge)| {
                    if matches!(edge, Edge::Unidirectional(_) | Edge::Bidirectional(_)) {
                        self.vertices.get(target_vid)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect()
        })
    }

    pub fn root_vid(&self) -> u32 {
        self.root_vid
    }

    pub fn set_vertex_payload(&mut self, vid: u32, payload: VP) -> bool {
        match self.vertices.get_mut(&vid) {
            Some(v) => { v.set_payload(payload); true }
            None => false,
        }
    }

    pub fn set_edge_payload(&mut self, from: u32, to: u32, payload: EP) -> bool {
        match self.edges.get_mut(&from) {
            Some(edges) => {
                match edges.iter_mut().find(|(t, _)| *t == to) {
                    Some((_, edge)) => { edge.set_payload(payload); true }
                    None => false,
                }
            }
            None => false,
        }
    }
}

// key = number, value = byte array; plugins on top interpret the bytes as
// domain-specific types (JSON, numpy array, pandas DataFrame, etc.)
pub struct Cursor<VP, EP> {
    current_node: u32,
    graph: Rc<RefCell<Graph<Vertex<VP>, Edge<EP>>>>,
    path: Vec<u32>,
    cache: std::collections::HashMap<u32, Vec<u8>>,
}

impl<VP: Clone, EP: Clone> Cursor<VP, EP> {
    pub fn new(graph: Rc<RefCell<Graph<Vertex<VP>, Edge<EP>>>>) -> Self {
        let root = graph.borrow().root_vid();
        Cursor {
            current_node: root,
            graph,
            path: vec![root],
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_graph(&self) -> Rc<RefCell<Graph<Vertex<VP>, Edge<EP>>>> {
        Rc::clone(&self.graph)
    }

    pub fn get_root(&self) -> u32 {
        self.graph.borrow().root_vid()
    }

    pub fn get_current_node(&self) -> u32 {
        self.current_node
    }

    pub fn get_node(&self) -> Option<Vertex<VP>> {
        self.graph.borrow().get_vertex(self.current_node).cloned()
    }

    pub fn get_edges(&self) -> Option<Vec<(u32, Edge<EP>)>> {
        self.graph.borrow().get_edges(self.current_node).cloned()
    }

    pub fn get_path(&self) -> Vec<u32> {
        self.path.clone()
    }

    pub fn move_to(&mut self, vid: u32) -> Option<u32> {
        let reachable = self.graph.borrow()
            .get_edges(self.current_node)
            .map(|edges| edges.iter().any(|(t, _)| *t == vid))
            .unwrap_or(false);
        if reachable {
            self.current_node = vid;
            self.path.push(vid);
            Some(vid)
        } else {
            None
        }
    }

    pub fn back(&mut self) -> Option<u32> {
        if self.path.len() > 1 {
            self.path.pop();
            let prev = *self.path.last().unwrap();
            self.current_node = prev;
            Some(prev)
        } else {
            None
        }
    }

    pub fn get_cache(&self) -> &std::collections::HashMap<u32, Vec<u8>> {
        &self.cache
    }

    pub fn get_cache_item(&self, key: u32) -> Option<&Vec<u8>> {
        self.cache.get(&key)
    }

    pub fn get_cache_item_mut(&mut self, key: u32) -> Option<&mut Vec<u8>> {
        self.cache.get_mut(&key)
    }

    pub fn remove_cache_item(&mut self, key: u32) -> Option<Vec<u8>> {
        self.cache.remove(&key)
    }

    pub fn contains_cache_item(&self, key: u32) -> bool {
        self.cache.contains_key(&key)
    }

    pub fn set_cache_item(&mut self, key: u32, value: Vec<u8>) {
        self.cache.insert(key, value);
    }

    pub fn get_cache_mut(&mut self) -> &mut std::collections::HashMap<u32, Vec<u8>> {
        &mut self.cache
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_graph() {
        let graph = sample_graph();
        let root_id = graph.root_vid;
        match graph.get_vertex(root_id) {
            Some(vertex) => {
                assert_eq!(vertex.label, "Start");
            }
            None => {
                panic!("Vertex with ID {} not found", root_id);
            }
        }
        println!("{:?}", graph);
        assert_eq!(4, 4);
    }

    #[test]
    fn cursor_moves() {
        let graph = Rc::new(RefCell::new(sample_graph()));
        let mut cursor = Cursor::new(Rc::clone(&graph));
        let available_vids = cursor
            .get_edges()
            .map(|edges| edges.iter().map(|e| e.0).collect::<Vec<u32>>());

        println!("Cursor pointing at node: {:?}", cursor.get_node());
        println!("Cursor node has edges: {:?}", cursor.get_edges());

        match cursor.move_to(available_vids.unwrap()[0]) {
            Some(vid) => {
                println!("Moved to node: {:?}", vid);
            }
            None => {
                panic!("Node not found");
            }
        }

        println!("Cursor pointing at node: {:?}", cursor.get_node());
        println!("Cursor node has edges: {:?}", cursor.get_edges());
    }

    fn sample_graph() -> Graph<Vertex, Edge> {
        let mut graph = Graph::new("Start");

        let root_id = graph.root_vid;

        let new_pricing_strategy_id = graph
            .add_child(
                &root_id,
                Vertex::new("New Price Strategy", None, None),
                None,
            )
            .unwrap();

        let pricing_distribution_id = graph
            .add_child(
                &new_pricing_strategy_id,
                Vertex::new("Pricing Distribution", None, None),
                None,
            )
            .unwrap();
        graph
            .add_child(
                &pricing_distribution_id,
                Vertex::new("Return Revenue", None, None),
                None,
            )
            .unwrap();
        graph
            .add_child(
                &pricing_distribution_id,
                Vertex::new("Set Min & Max", None, None),
                None,
            )
            .unwrap();

        graph
            .add_child(
                &new_pricing_strategy_id,
                Vertex::<()>::new("Overwrite", None, None),
                None,
            )
            .unwrap();

        graph.add_child(
            &root_id,
            Vertex::<()>::new("Adjust Live Pricing", None, None),
            None,
        );

        let end_live_experiment_id = graph
            .add_child(
                &root_id,
                Vertex::<()>::new("End Live Experiment", None, None),
                None,
            )
            .unwrap();
        graph.add_child(
            &end_live_experiment_id,
            Vertex::new("Roll out", None, None),
            None,
        );
        graph.add_child(
            &end_live_experiment_id,
            Vertex::new("Roll back", None, None),
            None,
        );
        graph
    }
}
