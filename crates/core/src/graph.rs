use crate::helper::get_rand_vid;
use fnv::FnvHashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Vertex<Payload = ()> {
    label: String,
    step: u32,
    payload: Option<Payload>,
}
impl<Payload> Vertex<Payload> {
    pub fn new(name: &str, payload: Option<Payload>, p_step: Option<u32>) -> Self {
        let step = p_step.unwrap_or(0);
        Self {
            label: name.to_string(),
            step: step + 1,
            payload,
        }
    }

    pub fn get_step(&self) -> u32 {
        self.step
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn get_payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }
}


#[derive(Debug)]
pub enum Edge<Payload = ()> {
    Unidirectional(Option<Payload>),
    Bidirectional(Option<Payload>),
}

#[derive(Debug)]
pub struct Graph<Vertex, Edge> {
    root_vid: Option<u32>,
    vertices: FnvHashMap<u32, Vertex>,
    edges: FnvHashMap<u32, Vec<(u32, Edge)>>,
}

impl<Payload> Graph<Vertex<Payload>, Edge<Payload>> {
    pub fn new(root_label: &str) -> Self {
        let mut g = Self {
            root_vid: None,
            vertices: FnvHashMap::default(),
            edges: FnvHashMap::default(),
        };
        let vid = get_rand_vid(2, u32::MAX);
        g.add_vertex(&vid, Vertex::<Payload>::new(root_label, None, None));
        g
    }

    /// Adds a vertex to the graph. If the graph is empty, the vertex is the root.
    pub fn add_vertex(&mut self, vid: &u32, vertex: Vertex<Payload>) -> u32 {
        if self.root_vid.is_none() {
            self.root_vid = Some(*vid);
        }
        self.vertices.insert(*vid, vertex);
        self.root_vid.unwrap()
    }

    pub fn add_child(
        &mut self,
        parent_vid: &u32,
        mut child: Vertex<Payload>,
        edge_type: Option<Edge<Payload>>,
    ) -> Option<u32> {
        let rand_vid = get_rand_vid(2, u32::MAX);

        if self.root_vid.is_none() {
            self.add_vertex(parent_vid, child);
            return Some(*parent_vid);
        }

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

    pub fn add_edge(&mut self, from: u32, to: u32, value: Edge<Payload>) {
        self.edges.entry(from).or_default().push((to, value));
    }

    pub fn get_vertex(&self, vid: u32) -> Option<&Vertex<Payload>> {
        self.vertices.get(&vid)
    }

    pub fn get_edges(&self, vid: u32) -> Option<&Vec<(u32, Edge<Payload>)>> {
        self.edges.get(&vid)
    }

    pub fn root_vid(&self) -> Option<u32> {
        self.root_vid
    }
}

struct Cursor<'a> {
    current_node: u32,
    g: Box<&'a Graph<Vertex, Edge>>,
}

impl<'a> Cursor<'a> {
    pub fn new(graph: &'a Graph<Vertex, Edge>) -> Self {
        Cursor {
            g: Box::new(graph),
            current_node: graph.root_vid.unwrap(),
        }
    }

    pub fn get_node(&self) -> Option<&Vertex> {
        self.g.get_vertex(self.current_node)
    }
    pub fn get_edges(&self) -> Option<&Vec<(u32, Edge)>> {
        self.g.get_edges(self.current_node)
    }

    pub fn move_to(&mut self, vid: u32) -> Option<u32> {
        let available_vids = self
            .get_edges()
            .map(|edges| edges.iter().map(|e| e.0).collect::<Vec<u32>>());
        match available_vids {
            Some(v) => {
                if v.contains(&vid) {
                    self.current_node = vid;
                    Some(vid)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_graph() {
        let graph = sample_graph();
        let root_id = graph.root_vid.unwrap();
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
        let graph = sample_graph();
        let mut cursor = Cursor::new(&graph);
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

        let root_id = graph.root_vid.unwrap();

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
