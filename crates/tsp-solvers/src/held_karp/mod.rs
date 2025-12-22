//! Held-Karp TSP solver implementation using branch-and-bound and Lagrangian relaxation.
//!
//! ## Call Structure of the Algorithm
//! The call structure of the algorithm and sub-methods is as follows. Indented functions indicate
//! that they are called by the function above them.
//!
//! - `held_karp`: Main entry point for the Held-Karp solver. Sets up parameters and initiates the
//!   branch-and-bound search.
//!     - `explore_node` Performs depth-first branch-and-bound search.
//!         - `explore_node` to recursively explore the search tree.
//!         - `edge_to_branch_on` to select edges for branching.
//!         - `held_karp_lower_bound` to compute lower bounds using 1-trees.
//!             - `min_one_tree` to compute minimum 1-trees as part of the lower bound calculation.
//!                 - `min_spanning_tree` to compute minimum spanning trees using Prim's algorithm.
//!
//! The basic idea of the Held-Karp algorithm is to compute lower bounds on the TSP tour cost using
//! 1-trees and Lagrangian relaxation.
//!
//! ## 1-trees
//!
//! 1-trees are minimum spanning trees that span nodes 2 to n, plus two minimum cost edges
//! connecting node 1 to the tree. This is always a lower bound on the cost of a TSP tour, since
//! any TSP tour is a 1-tree. Thus, the cheapest 1-tree provides a lower bound on the TSP tour cost.
//!
//!
//! ## Lagrangian Relaxation
//!
//! Because the computed 1-tree might have many nodes with degree unequal to 2, we introduce
//! penalties for each node based on how far their degree is from 2. This is what we call Lagrangian
//! relaxation. By iteratively adjusting the penalties based on the degree of nodes in the 1-tree,
//! we can converge towards a tighter lower bound on the TSP tour cost. Once an actual tour is
//! found, we can use that as an upper bound to prune the search space in the branch-and-bound
//! exploration.
//!
//! ## Edge States
//!
//! Edges can be in one of three states: Available, Excluded, or Fixed. This allows the
//! branch-and-bound search to systematically explore different configurations of the TSP tour
//! by forcibly including or excluding edges.

use std::u32;

use tsp_core::instance::{
    UnTour,
    edge::{UnEdge, data::EdgeDataMatrixSym, distance::Distance},
};

use crate::held_karp::{fixed_point_arithmetic::ScaledDistance, trees::min_one_tree};

mod fixed_point_arithmetic;
mod trees;

pub fn held_karp(
    distances: &EdgeDataMatrixSym<Distance>,
    edge_states: &mut EdgeStateMatrix,
    upper_bound: u32,
) {
    let mut edge_states = EdgeDataMatrixSym {
        data: vec![EdgeState::Available; distances.data.len()],
        dimension: distances.dimension,
    };

    explore_node(distances, &mut edge_states, upper_bound, &mut 0, None, 0);
}

const INITIAL_MAX_ITERATIONS: usize = 1_000;
const MAX_ITERATIONS: usize = 10;

const INITIAL_ALPHA: f64 = 2.0;

const INITIAL_BETA: f64 = 0.99;
const BETA_INCREASE: f64 = 0.9;

type EdgeStateMatrix = EdgeDataMatrixSym<EdgeState>;

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeState {
    Available = 1,
    Excluded = 0,
    Fixed = -1,
}

impl EdgeState {
    /// Constructs an EdgeState from an i8 value, panicking if the value is invalid.
    fn from_i8(value: i8) -> Self {
        match value {
            1 => EdgeState::Available,
            0 => EdgeState::Excluded,
            -1 => EdgeState::Fixed,
            _ => panic!("Invalid value for EdgeState: {}", value),
        }
    }

    /// Constructs an EdgeState from an i8 value without checking its validity.
    ///
    /// # Safety
    /// The caller must ensure that the value is one of 1, 0, or -1.
    unsafe fn from_i8_unchecked(value: i8) -> Self {
        // SAFETY: Caller guarantees value is valid
        unsafe { std::mem::transmute(value) }
    }

    fn flip(self) -> Self {
        // SAFETY: The negation of a valid EdgeState value is also valid
        unsafe { EdgeState::from_i8_unchecked(-(self as i8)) }
    }
}

/// Depth-first branch-and-bound search to find optimal TSP Tour.
///
/// TODO: Document properly
///
/// bb_counter: A mutable reference to a usize that counts the number of branch-and-bound nodes
/// explored
/// bb_limit: A usize that sets the limit for branch-and-bound exploration
/// depth: The current depth in the search tree
/// max_iterations: The maximum number of iterations allowed
/// upper_bound: A mutable reference to the current best upper bound on the tour cost
fn explore_node(
    distances: &EdgeDataMatrixSym<Distance>,
    edge_states: &mut EdgeStateMatrix,
    upper_bound: u32,
    bb_counter: &mut usize,
    bb_limit: Option<usize>,
    depth: usize,
) {
    // Increment the branch count
    *bb_counter += 1;

    // Check if the branch and bound limit has been reached
    if let Some(limit) = bb_limit {
        if *bb_counter >= limit {
            return;
        }
    }

    // match held_karp_lower_bound() {
    //     LowerBoundOutput::Tour(cost) => {
    //         // Found a new tour, that is, an upper bound
    //         upper_bound = cost;
    //         return;
    //     }
    //     LowerBoundOutput::LowerBound(lower_bound) => {
    //         // Check if the lower bound is better than the current best cost
    //         if lower_bound < upper_bound {
    //             // Prune this node, as we have already found a better tour than the lower bound
    //             return;
    //         }
    //     }
    // };

    let branching_edge = edge_to_branch_on();

    // Try exploring the branch excluding the edge
    todo!();

    // Try exploring the branch including the edge
    todo!();
}

enum LowerBoundOutput {
    LowerBound(Distance),
    Tour(UnTour),
}

/// Compute Held-Karp lower bound using 1-trees and Lagrangian relaxation
fn held_karp_lower_bound(
    distances: &EdgeDataMatrixSym<Distance>,
    scaled_distances: &EdgeDataMatrixSym<ScaledDistance>,
    edge_states: &EdgeStateMatrix,
    node_penalties: &mut [ScaledDistance],
    upper_bound: Distance,
    max_iterations: usize,
    beta: f64,
) -> Option<LowerBoundOutput> {
    let scaled_bound = ScaledDistance::from_distance(upper_bound);

    // Tracks the current best lower bound found
    let mut scaled_best_lower_bound = ScaledDistance::MIN;

    let mut iter_count = 0;

    let mut alpha = INITIAL_ALPHA;

    let node_penalty_sum: ScaledDistance = node_penalties.iter().sum();

    loop {
        let one_tree = min_one_tree(scaled_distances, edge_states, node_penalties)?;

        // Compute the cost of the 1-tree with penalties. This is simultaneously the value of
        // the lagrangian relaxation and thus a lower bound (possibly an upper bound too, if it is a
        // tour).
        let one_tree_cost = {
            let mut base_cost = 2 * node_penalty_sum;

            for edge in &one_tree {
                base_cost += scaled_distances.get_data(edge.from, edge.to);
                base_cost -= node_penalties[edge.from.0];
                base_cost -= node_penalties[edge.to.0];
            }

            base_cost
        };

        if one_tree_cost > scaled_best_lower_bound {
            scaled_best_lower_bound = one_tree_cost;
        }

        if one_tree_cost >= scaled_bound {
            // Lower bound exceeds current upper bound, prune
            break;
        }

        // Next we check the degrees of the nodes in the 1-tree
        // Deg[node] can be interpreted as follows:
        //  Deg[node] < 0: Node has degree > 2 -> we need to decrease its penalty. This makes edges
        //                 incident to node more expensive, that is, less likely to be selected.
        //  Deg[node] > 0: Node has degree < 2 -> we need to increase its penalty. This makes edges
        //                 incident to node cheaper, that is, more likely to be selected.
        //  Deg[node] == 0: Node has degree == 2 -> no change to penalty.
        let mut deg = vec![2i32; distances.dimension];

        for edge in &one_tree {
            deg[edge.from.0] -= 1;
            deg[edge.to.0] -= 1;
        }

        let square_sum = deg.iter().map(|&d| d * d).sum::<i32>();

        if square_sum == 0 {
            // Found a tour
            let cost: Distance = one_tree
                .iter()
                .map(|edge| distances.get_data(edge.from, edge.to))
                .sum();

            return Some(LowerBoundOutput::Tour(UnTour {
                edges: one_tree,
                cost,
            }));
        }

        // We have not found a tour yet, so we want to update the penalties
        iter_count += 1;

        if iter_count >= max_iterations {
            // Reached maximum iterations
            break;
        }

        // TODO: Research on subgradient method for non-smooth optimization to find out more about
        // this
        let step_size =
            (alpha * ((scaled_bound.0 - one_tree_cost.0) as f64 / (square_sum as f64))) as i32;

        if step_size <= 3 {
            // Step size is very small (<= 3 in scaled), we probably won't be making much progress
            break;
        }

        alpha *= beta;

        // Update penalties based on degree deviations and step size
        // TODO: Handle overflows
        for (node_penalty, &d) in node_penalties.iter_mut().zip(deg.iter()) {
            let adjustment = ScaledDistance(step_size * d);
            *node_penalty += adjustment;
        }
    }

    let best_lower_bound = scaled_best_lower_bound.to_distance_rounded_up();

    Some(LowerBoundOutput::LowerBound(best_lower_bound))
}

fn edge_to_branch_on() {
    todo!();
}
