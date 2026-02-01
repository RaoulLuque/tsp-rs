use crate::instance::node::Node;

/// An undirected edge between two nodes.
#[derive(Debug, Clone, Copy)]
pub struct UnEdge {
    pub from: Node,
    pub to: Node,
}

impl UnEdge {
    pub fn new(from: Node, to: Node) -> Self {
        Self { from, to }
    }
}

impl PartialEq for UnEdge {
    fn eq(&self, other: &Self) -> bool {
        (self.from == other.from && self.to == other.to)
            || (self.from == other.to && self.to == other.from)
    }
}

impl Eq for UnEdge {}

impl PartialOrd for UnEdge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnEdge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let (min_self, max_self) = if self.from <= self.to {
            (self.from, self.to)
        } else {
            (self.to, self.from)
        };
        let (min_other, max_other) = if other.from <= other.to {
            (other.from, other.to)
        } else {
            (other.to, other.from)
        };

        match min_self.cmp(&min_other) {
            std::cmp::Ordering::Equal => max_self.cmp(&max_other),
            ord => ord,
        }
    }
}

impl From<(Node, Node)> for UnEdge {
    fn from(value: (Node, Node)) -> Self {
        UnEdge::new(value.0, value.1)
    }
}

impl From<(usize, usize)> for UnEdge {
    fn from(value: (usize, usize)) -> Self {
        UnEdge::new(Node(value.0), Node(value.1))
    }
}
