use crate::instance::node::Node;

/// An undirected edge between two nodes.
///
/// Derives [PartialEq], [Eq], [PartialOrd], and [Ord] such that the edge (A, B) is considered
/// equal to (B, A), and ordering is based on the smaller node first.
#[derive(Debug, Clone, Copy)]
pub struct UnEdge {
    /// The first node of the edge.
    pub from: Node,
    /// The second node of the edge.
    pub to: Node,
}

impl UnEdge {
    /// Creates a new undirected edge between two nodes.
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
