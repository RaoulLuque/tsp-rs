#[derive(Debug, Clone, Copy)]
pub struct UnEdge {
    pub from: usize,
    pub to: usize,
}

impl UnEdge {
    pub fn new(from: usize, to: usize) -> Self {
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

#[derive(Debug)]
/// An undirected edge with an inverse weight for use in a max-heap.
///
/// That is, when comparing two edges, the one with the lower cost is considered greater.
pub struct InverseWeightedUnEdge {
    pub cost: u32,
    pub from: usize,
    pub to: usize,
}

impl InverseWeightedUnEdge {
    pub fn new(cost: u32, from: usize, to: usize) -> Self {
        Self { cost, from, to }
    }

    pub fn to_edge(&self) -> UnEdge {
        UnEdge {
            from: self.from,
            to: self.to,
        }
    }
}

impl PartialEq for InverseWeightedUnEdge {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for InverseWeightedUnEdge {}

impl PartialOrd for InverseWeightedUnEdge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.cost.cmp(&self.cost))
    }
}

impl Ord for InverseWeightedUnEdge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}
