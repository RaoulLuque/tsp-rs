pub fn held_karp() {
    todo!();
}

const INITIAL_MAX_ITERATIONS: usize = 1_000;
const MAX_ITERATIONS: usize = 10;

const INITIAL_BETA: f64 = 0.99;
const BETA_INCREASE: f64 = 0.9;

/// Depth-first branch-and-bound search to find optimal TSP Tour.
///
/// TODO: Document properly
///
/// bb_counter: A mutable reference to a usize that counts the number of branch-and-bound nodes
/// explored bb_limit: A usize that sets the limit for branch-and-bound exploration
/// depth: The current depth in the search tree
/// max_iterations: The maximum number of iterations allowed
fn explore_node(
    upper_bound: &mut u32,
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

    match held_karp_lower_bound() {
        LowerBoundOutput::Tour(cost) => {
            // Found a new tour, that is, an upper bound
            *upper_bound = cost;
            return;
        }
        LowerBoundOutput::LowerBound(lower_bound) => {
            // Check if the lower bound is better than the current best cost
            if lower_bound < *upper_bound {
                // Prune this node, as we have already found a better tour than the lower bound
                return;
            }
        }
    };

    let branching_edge = edge_to_branch_on();

    // Try exploring the branch excluding the edge
    todo!();

    // Try exploring the branch including the edge
    todo!();
}

enum LowerBoundOutput {
    LowerBound(u32),
    Tour(u32),
}

/// Compute Held-Karp lower bound using 1-trees and Lagrangian relaxation
fn held_karp_lower_bound() -> LowerBoundOutput {
    todo!();
}

fn edge_to_branch_on() {
    todo!();
}
