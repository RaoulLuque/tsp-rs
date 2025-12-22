use std::collections::BinaryHeap;

use tsp_core::instance::{
    edge::{
        InvWeightUnEdge, UnEdge,
        data::{EdgeDataMatrix, EdgeDataMatrixSym},
        distance::{Distance, DistanceMatrixSym},
    },
    node::Node,
};

use crate::{
    CustomBitVec,
    held_karp::{EdgeState, fixed_point_arithmetic::ScaledDistance},
};

/// Compute a minimum 1-tree with given node penalties and edge states.
///
/// Note that the singled out node in this implementation is the last node opposed to the first
/// node, as in some other implementations.
pub fn min_one_tree(
    distances_scaled: &EdgeDataMatrixSym<ScaledDistance>,
    edge_states: &EdgeDataMatrixSym<EdgeState>,
    penalties: &[ScaledDistance],
) -> Option<Vec<UnEdge>> {
    // First, compute the minimum spanning tree on all nodes except the last node
    let distances_restricted = distances_scaled.restrict_to_first_n(distances_scaled.dimension - 1);
    let edge_states_restricted = edge_states.restrict_to_first_n(edge_states.dimension - 1);
    let tree = min_spanning_tree(&distances_restricted, &edge_states_restricted, penalties)?;

    // Next, find the two cheapest edges connecting the last node to the tree
    let last_node = Node(distances_scaled.dimension() - 1);
    // We will uphold the following invariant dist_cheapest_edge_a <= dist_cheapest_edge_b
    let mut dist_cheapest_edge_a = ScaledDistance::MAX;
    let mut dist_cheapest_edge_b = ScaledDistance::MAX;
    let mut cheapest_neighbor_a = None;
    let mut cheapest_neighbor_b = None;

    for node_index in 0..(distances_scaled.dimension() - 1) {
        let node = Node(node_index);
        match edge_states.get_data(last_node, node) {
            EdgeState::Excluded => continue,
            EdgeState::Available => {
                let distance = distances_scaled.get_data(last_node, node);
                if distance < dist_cheapest_edge_a {
                    // Assign new value to cheapest edge a, and move previous a to b
                    // (because of the invariant)
                    dist_cheapest_edge_b = dist_cheapest_edge_a;
                    cheapest_neighbor_b = cheapest_neighbor_a;
                    dist_cheapest_edge_a = distance;
                    cheapest_neighbor_a = Some(node);
                } else if distance < dist_cheapest_edge_b {
                    // Cheaper than b but not a, so just update b
                    dist_cheapest_edge_b = distance;
                    cheapest_neighbor_b = Some(node);
                }
            }
            EdgeState::Fixed => {
                if dist_cheapest_edge_b == ScaledDistance::MIN {
                    // By the invariant, this implies that dist_cheapest_edge_a is also
                    // Distance(i32::MIN), meaning we have already included two
                    // fixed edges and just found another one, that is, we are infeasible.
                    return None;
                }

                // Proceed same as EdgeState::Available && distance < dist_cheapest_edge_a
                dist_cheapest_edge_b = dist_cheapest_edge_a;
                cheapest_neighbor_b = cheapest_neighbor_a;
                dist_cheapest_edge_a = ScaledDistance::MIN;
                cheapest_neighbor_a = Some(node);
            }
        }
    }

    if let Some(neighbor_b) = cheapest_neighbor_b {
        let mut one_tree = tree;
        let neighbor_a =
            cheapest_neighbor_a.expect("Cheapest neighbor A should exist by invariant");
        one_tree.push(UnEdge::new(last_node, neighbor_a));
        one_tree.push(UnEdge::new(last_node, neighbor_b));
        Some(one_tree)
    } else {
        // If neighbor_b does not exist, we were unable to find two edges to connect the last node,
        // so the 1-tree is not feasible.
        None
    }
}

/// Compute a minimum spanning tree with given edge states and node penalties. Implements a
/// variation of Prim's algorithm to abide the edge states.
///
/// Returns a vector of edges representing the minimum spanning tree.
///
/// For more details, see https://en.wikipedia.org/wiki/Prim%27s_algorithm
fn min_spanning_tree(
    scaled_distances: &impl EdgeDataMatrix<ScaledDistance>,
    edge_states: &impl EdgeDataMatrix<EdgeState>,
    penalties: &[ScaledDistance],
) -> Option<Vec<UnEdge>> {
    let number_of_nodes = scaled_distances.dimension();

    // Track which nodes are yet to be added to the tree
    let mut remaining_nodes = Vec::with_capacity(number_of_nodes);
    for node_index in 1..number_of_nodes {
        remaining_nodes.push(Node(node_index));
    }

    // For each node, track the best predecessor node and cost to reach it (Initialize with
    // unreachable values)
    let mut best_pred_to_node = vec![Node(number_of_nodes + 1); number_of_nodes];
    let mut best_cost_to_node = vec![ScaledDistance::MAX; number_of_nodes];

    // Start from node 0
    let mut curr = Node(0);

    // The resulting tree edges in no particular order
    let mut tree = Vec::with_capacity(number_of_nodes - 1);

    // Tree contains n - 1 edges
    for _ in 0..(number_of_nodes - 1) {
        let mut cheapest_edge = ScaledDistance::MAX;
        let mut cheapest_node = None;

        let current_penalty = penalties[curr.0];

        for (index, next) in remaining_nodes.iter().enumerate() {
            match edge_states.get_data(curr, *next) {
                EdgeState::Excluded => continue,
                EdgeState::Available => {
                    let distance = scaled_distances.get_data(curr, *next);
                    let adjusted_distance = distance - current_penalty - penalties[next.0];
                    if adjusted_distance < best_cost_to_node[next.0] {
                        best_cost_to_node[next.0] = adjusted_distance;
                        best_pred_to_node[next.0] = curr;
                    }
                }
                EdgeState::Fixed => {
                    // The edge is fixed, so we must include it in the tree
                    if best_cost_to_node[next.0] == ScaledDistance::MIN {
                        // This means we have already included the node next via a fixed edge, so
                        // including it again would create a cycle. Therefore, the MST is not
                        // possible with the current (fixed) edge states.
                        return None;
                    }

                    // Force this edge by setting its cost to the minimum possible value
                    best_cost_to_node[next.0] = ScaledDistance::MIN;
                    best_pred_to_node[next.0] = curr;
                }
            }

            if best_cost_to_node[next.0] < cheapest_edge {
                cheapest_edge = best_cost_to_node[next.0];
                cheapest_node = Some((index, *next));
            }
        }

        // Add the cheapest edge to the tree
        if let Some((index, cheapest_node)) = cheapest_node {
            tree.push(UnEdge::new(
                best_pred_to_node[cheapest_node.0],
                cheapest_node,
            ));
            remaining_nodes.swap_remove(index);
            curr = cheapest_node;
        } else {
            // We were unable to reach the remaining nodes, so the MST with the current edge states
            // is not feasible.
            return None;
        }
    }

    debug_assert_eq!(tree.len(), number_of_nodes - 1);

    Some(tree)
}

#[cfg(test)]
mod tests {

    use tsp_core::instance::edge::data::EdgeDataMatrixSym;

    use super::*;

    #[test]
    fn test_min_spanning_tree_simple_tree() {
        let distance_matrix = EdgeDataMatrixSym::slow_new_from_distance_function(10, |from, to| {
            if from.0 + 1 == to.0 || from.0 == to.0 + 1 {
                ScaledDistance(0)
            } else {
                ScaledDistance(1)
            }
        });
        let penalties = vec![ScaledDistance(0); 10];
        let edge_states = EdgeDataMatrixSym {
            data: vec![EdgeState::Available; distance_matrix.data.len()],
            dimension: distance_matrix.dimension,
        };

        let mst = min_spanning_tree(&distance_matrix, &edge_states, &penalties).unwrap();
        assert_eq!(mst.len(), 9);
        let expected = (0..9)
            .map(|i| UnEdge {
                from: Node(i),
                to: Node(i + 1),
            })
            .collect::<Vec<_>>();
        mst.iter().for_each(|edge| {
            assert!(
                expected.contains(edge),
                "Edge {:?} not in expected MST",
                edge
            );
        });
    }

    #[test]
    fn test_min_spanning_tree_excluded_infeasible() {
        let distance_matrix =
            EdgeDataMatrixSym::new_from_dimension_with_value(10, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); 10];
        let edge_states = EdgeDataMatrixSym {
            data: vec![EdgeState::Excluded; distance_matrix.data.len()],
            dimension: distance_matrix.dimension,
        };

        let mst = min_spanning_tree(&distance_matrix, &edge_states, &penalties);
        assert_eq!(mst, None);
    }

    #[test]
    fn test_min_spanning_tree_infeasible_node_isolated() {
        let distance_matrix =
            EdgeDataMatrixSym::new_from_dimension_with_value(5, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); 5];
        let mut edge_states = Vec::with_capacity(distance_matrix.data.len());
        for from in 0..5 {
            for to in 0..=from {
                if (from == 2) || (to == 2) {
                    edge_states.push(EdgeState::Excluded);
                } else if (from + 1 == to) || (to + 1 == from) {
                    edge_states.push(EdgeState::Fixed);
                } else {
                    edge_states.push(EdgeState::Available);
                }
            }
        }

        let edge_states = EdgeDataMatrixSym {
            data: edge_states,
            dimension: distance_matrix.dimension,
        };

        let mst = min_spanning_tree(&distance_matrix, &edge_states, &penalties);
        assert_eq!(mst, None);
    }

    #[test]
    fn test_min_spanning_tree_fixed() {
        let distance_matrix =
            EdgeDataMatrixSym::new_from_dimension_with_value(5, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); 5];
        let mut edge_states = Vec::with_capacity(distance_matrix.data.len());
        for from in 0..5 {
            for to in 0..=from {
                if (from + 1 == to) || (to + 1 == from) {
                    edge_states.push(EdgeState::Fixed);
                } else {
                    edge_states.push(EdgeState::Available);
                }
            }
        }

        let edge_states = EdgeDataMatrixSym {
            data: edge_states,
            dimension: distance_matrix.dimension,
        };

        let mst = min_spanning_tree(&distance_matrix, &edge_states, &penalties).unwrap();
        let expected = vec![
            UnEdge::new(Node(0), Node(1)),
            UnEdge::new(Node(1), Node(2)),
            UnEdge::new(Node(2), Node(3)),
            UnEdge::new(Node(3), Node(4)),
        ];
        assert_eq!(mst.len(), expected.len());
        mst.iter().for_each(|edge| {
            assert!(
                expected.contains(edge),
                "Edge {:?} not in expected MST",
                edge
            );
        });
    }
}
