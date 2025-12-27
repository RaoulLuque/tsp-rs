use tsp_core::instance::{
    edge::{
        UnEdge,
        data::{EdgeDataMatrix, EdgeDataMatrixZeroRemoved},
        distance::ScaledDistance,
    },
    node::Node,
};

use crate::held_karp::EdgeState;

/// Compute a minimum 1-tree with given node penalties and edge states.
///
/// Note that the singled out node in this implementation is the last node opposed to the first
/// node, as in some other implementations.
pub fn min_one_tree(
    distances_scaled: &EdgeDataMatrix<ScaledDistance>,
    edge_states: &EdgeDataMatrix<EdgeState>,
    penalties: &[ScaledDistance],
) -> Option<Vec<UnEdge>> {
    let (distances_scaled_zero, distances_scaled_rest) = distances_scaled.split_first_row();
    let (edge_states_zero, edge_states_rest) = edge_states.split_first_row();

    // First, compute the minimum spanning tree on all nodes except the last node
    let tree = min_spanning_tree(distances_scaled_rest, edge_states_rest, penalties)?;

    // Next, find the two cheapest edges connecting the last node to the tree
    let node_zero = Node(0);
    // We will uphold the following invariant dist_cheapest_edge_a <= dist_cheapest_edge_b
    let mut dist_cheapest_edge_a = ScaledDistance::MAX;
    let mut dist_cheapest_edge_b = ScaledDistance::MAX;
    let mut cheapest_neighbor_a = None;
    let mut cheapest_neighbor_b = None;

    let mut zero_neighbors_iter = distances_scaled_zero.iter().enumerate();
    // Start from index 1, as index 0 is the distance from node 0
    zero_neighbors_iter.next();

    for (node_index, &distance) in zero_neighbors_iter {
        let node = Node(node_index);
        match edge_states_zero[node_index] {
            EdgeState::Excluded => continue,
            EdgeState::Available => {
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
        one_tree.push(UnEdge::new(node_zero, neighbor_a));
        one_tree.push(UnEdge::new(node_zero, neighbor_b));
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
    distances_scaled: EdgeDataMatrixZeroRemoved<ScaledDistance>,
    edge_states: EdgeDataMatrixZeroRemoved<EdgeState>,
    penalties: &[ScaledDistance],
) -> Option<Vec<UnEdge>> {
    let number_of_nodes_in_tree = distances_scaled.dimension_adjusted();
    // Track which nodes are yet to be added to the tree
    let mut remaining_nodes = Vec::with_capacity(number_of_nodes_in_tree);
    for node_index in 2..=number_of_nodes_in_tree {
        remaining_nodes.push(Node(node_index));
    }

    // For each node, track the best predecessor node and cost to reach it (Initialize with
    // unreachable values)
    let mut best_pred_to_node =
        vec![Node(number_of_nodes_in_tree + 1); number_of_nodes_in_tree + 1];
    let mut best_cost_to_node = vec![ScaledDistance::MAX; number_of_nodes_in_tree + 1];

    // Start from node 1
    let mut curr = Node(1);

    // The resulting tree edges in no particular order
    let mut tree = Vec::with_capacity(number_of_nodes_in_tree - 1);

    // Tree contains n - 1 edges
    for _ in 0..(number_of_nodes_in_tree - 1) {
        let mut cheapest_edge = ScaledDistance::MAX;
        let mut cheapest_node = None;

        let current_penalty = penalties[curr.0];
        let distances_scaled_curr = distances_scaled.get_adjacency_list(curr);
        let edge_states_curr = edge_states.get_adjacency_list(curr);

        for (index, next) in remaining_nodes.iter().enumerate() {
            match edge_states_curr[next.0] {
                EdgeState::Excluded => continue,
                EdgeState::Available => {
                    let distance = distances_scaled_curr[next.0];
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

    debug_assert_eq!(tree.len(), number_of_nodes_in_tree - 1);

    Some(tree)
}

#[cfg(test)]
mod tests {

    use tsp_core::instance::edge::data::EdgeDataMatrix;

    use super::*;

    #[test]
    fn test_min_spanning_tree_simple_tree() {
        let dimension = 11;
        let distance_matrix =
            EdgeDataMatrix::slow_new_from_distance_function(dimension, |from, to| {
                if from.0 + 1 == to.0 || from.0 == to.0 + 1 {
                    ScaledDistance(0)
                } else {
                    ScaledDistance(1)
                }
            });
        let penalties = vec![ScaledDistance(0); dimension];
        let edge_states = EdgeDataMatrix {
            data: vec![EdgeState::Available; distance_matrix.data.len()],
            dimension: distance_matrix.dimension,
        };
        let (_, distance_matrix_rest) = distance_matrix.split_first_row();
        let (_, edge_states_rest) = edge_states.split_first_row();

        let mst = min_spanning_tree(distance_matrix_rest, edge_states_rest, &penalties).unwrap();
        assert_eq!(mst.len(), dimension - 2);
        let expected = (1..(dimension))
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
        let distance_matrix = EdgeDataMatrix::new_from_dimension_with_value(10, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); 10];
        let edge_states = EdgeDataMatrix {
            data: vec![EdgeState::Excluded; distance_matrix.data.len()],
            dimension: distance_matrix.dimension,
        };
        let (_, distance_matrix_rest) = distance_matrix.split_first_row();
        let (_, edge_states_rest) = edge_states.split_first_row();

        let mst = min_spanning_tree(distance_matrix_rest, edge_states_rest, &penalties);
        assert_eq!(mst, None);
    }

    #[test]
    fn test_min_spanning_tree_infeasible_node_isolated() {
        let dimension = 6;
        let distance_matrix =
            EdgeDataMatrix::new_from_dimension_with_value(dimension, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); dimension];
        let mut edge_states =
            EdgeDataMatrix::new_from_dimension_with_value(dimension, EdgeState::Available);
        for from in 0..dimension {
            for to in 0..=from {
                if (from == 2) || (to == 2) {
                    edge_states.set_data(Node(from), Node(to), EdgeState::Excluded);
                } else if to + 1 == from {
                    edge_states.set_data(Node(from), Node(to), EdgeState::Fixed);
                } else {
                    edge_states.set_data(Node(from), Node(to), EdgeState::Available);
                }
            }
        }

        let (_, distance_matrix_rest) = distance_matrix.split_first_row();
        let (_, edge_states_rest) = edge_states.split_first_row();

        let mst = min_spanning_tree(distance_matrix_rest, edge_states_rest, &penalties);
        assert_eq!(mst, None);
    }

    #[test]
    fn test_min_spanning_tree_fixed() {
        let dimension = 6;
        let distance_matrix =
            EdgeDataMatrix::new_from_dimension_with_value(dimension, ScaledDistance(0));
        let penalties = vec![ScaledDistance(0); dimension];
        let mut edge_states =
            EdgeDataMatrix::new_from_dimension_with_value(dimension, EdgeState::Available);
        for from in 0..dimension {
            for to in 0..=from {
                if to + 1 == from {
                    edge_states.set_data(Node(from), Node(to), EdgeState::Fixed);
                } else {
                    edge_states.set_data(Node(from), Node(to), EdgeState::Available);
                }
            }
        }

        let (_, distance_matrix_rest) = distance_matrix.split_first_row();
        let (_, edge_states_rest) = edge_states.split_first_row();

        let mst = min_spanning_tree(distance_matrix_rest, edge_states_rest, &penalties).unwrap();
        let expected = vec![
            UnEdge::new(Node(2), Node(3)),
            UnEdge::new(Node(3), Node(4)),
            UnEdge::new(Node(1), Node(2)),
            UnEdge::new(Node(4), Node(5)),
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
