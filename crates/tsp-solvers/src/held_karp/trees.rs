use std::collections::BinaryHeap;

use tsp_core::instance::{
    edge::{
        InvWeightUnEdge, UnEdge,
        data::EdgeDataMatrix,
        distance::{Distance, DistanceMatrixSym},
    },
    node::Node,
};

use crate::{CustomBitVec, held_karp::EdgeState};

/// Compute a minimum 1-tree with given node penalties
fn min_one_tree(
    distances: DistanceMatrixSym,
    edge_states: &impl EdgeDataMatrix<EdgeState>,
    penalties: &[i32],
) {
    let distances_restricted = distances.restrict_to_first_n(distances.dimension - 1);
    let tree = min_spanning_tree(&distances_restricted, edge_states, penalties);
}

/// Compute a minimum spanning tree for given nodes and edges using prim's algorithm.
/// We assume that the distance matrix is valid, i.e., it models a complete graph.
///
/// Returns a vector of number of nodes - 1 edges representing the minimum spanning tree.
///
/// For more details, see https://en.wikipedia.org/wiki/Prim%27s_algorithm
fn min_spanning_tree(
    distances: &impl EdgeDataMatrix<Distance>,
    edge_states: &impl EdgeDataMatrix<EdgeState>,
    penalties: &[i32],
) -> Option<Vec<UnEdge>> {
    let number_of_nodes = distances.dimension();

    // Track which nodes are yet to be added to the tree
    let mut remaining_nodes = Vec::with_capacity(number_of_nodes);
    for node_index in 1..number_of_nodes {
        remaining_nodes.push(Node(node_index));
    }

    // For each node, track the best predecessor node and cost to reach it (Initialize with
    // unreachable values)
    let mut best_pred_to_node = vec![Node(number_of_nodes + 1); number_of_nodes];
    let mut best_cost_to_node = vec![Distance(i32::MAX); number_of_nodes];

    // Start from node 0
    let mut curr = Node(0);

    // The resulting tree edges in no particular order
    let mut tree = Vec::with_capacity(number_of_nodes - 1);

    // Tree contains n - 1 edges
    for _ in 0..(number_of_nodes - 1) {
        let mut cheapest_edge = Distance(i32::MAX);
        let mut cheapest_node = None;

        let current_penalty = penalties[curr.0];

        for (index, next) in remaining_nodes.iter().enumerate() {
            match edge_states.get_data(curr, *next) {
                EdgeState::Excluded => continue,
                EdgeState::Available => {
                    let distance = distances.get_data(curr, *next);
                    let adjusted_distance =
                        Distance(distance.0 - current_penalty - penalties[next.0]);
                    if adjusted_distance < best_cost_to_node[next.0] {
                        best_cost_to_node[next.0] = adjusted_distance;
                        best_pred_to_node[next.0] = curr;
                    }
                }
                EdgeState::Fixed => {
                    // The edge is fixed, so we must include it in the tree
                    if best_cost_to_node[next.0] == Distance(i32::MIN) {
                        // This means we have already included the node next via a fixed edge, so
                        // including it again would create a cycle. Therefore, the MST is not
                        // possible with the current (fixed) edge states.
                        return None;
                    }

                    // Force this edge by setting its cost to the minimum possible value
                    best_cost_to_node[next.0] = Distance(i32::MIN);
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

    Some(tree)
}

#[cfg(test)]
mod tests {

    use tsp_core::instance::edge::data::EdgeDataMatrixSym;

    use super::*;

    #[test]
    fn test_min_spanning_tree_simple_tree() {
        let distance_matrix = DistanceMatrixSym::slow_new_from_distance_function(10, |from, to| {
            if from.0 + 1 == to.0 || from.0 == to.0 + 1 {
                Distance(0)
            } else {
                Distance(1)
            }
        });
        let penalties = vec![0; 10];
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
        let distance_matrix = DistanceMatrixSym::new_from_dimension_with_value(10, Distance(0));
        let penalties = vec![0; 10];
        let edge_states = EdgeDataMatrixSym {
            data: vec![EdgeState::Excluded; distance_matrix.data.len()],
            dimension: distance_matrix.dimension,
        };

        let mst = min_spanning_tree(&distance_matrix, &edge_states, &penalties);
        assert_eq!(mst, None);
    }

    #[test]
    fn test_min_spanning_tree_infeasible_node_isolated() {
        let distance_matrix = DistanceMatrixSym::new_from_dimension_with_value(5, Distance(0));
        let penalties = vec![0; 5];
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
        let distance_matrix = DistanceMatrixSym::new_from_dimension_with_value(5, Distance(0));
        let penalties = vec![0; 5];
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
