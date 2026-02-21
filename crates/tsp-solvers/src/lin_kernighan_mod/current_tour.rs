use tsp_core::instance::node::Node;

pub(super) struct CurrentTour {
    nodes: Vec<Node>,
}

impl CurrentTour {
    pub(super) fn successor(&self, node: Node) -> Node {
        let index = self
            .nodes
            .iter()
            .position(|&n| n == node)
            .expect("Node should be in the current tour");
        self.nodes[(index + 1) % self.nodes.len()]
    }
}
