use std::{
    sync::{Arc, Mutex},
    thread,
};

use log::{debug, info, trace};
use tsp_core::instance::{
    UnTour,
    distance::{Distance, ScaledDistance},
    edge::UnEdge,
    matrix::Matrix,
    node::Node,
};

use crate::held_karp_mod::{
    BETA, EdgeState, INITIAL_ALPHA, INITIAL_BETA, INITIAL_MAX_ITERATIONS, MAX_ITERATIONS,
    edge_to_branch_on, initial_penalties, min_one_tree,
};

///  TODO: Adapt documentation
///
///  Solve the Traveling Salesman Problem using the Held-Karp algorithm.
///
/// For a detailed explanation of the algorithm, see the [module-level
/// documentation][crate::held_karp_mod].
pub fn held_karp_parallel(distances: &Matrix<Distance>) -> Option<UnTour> {
    info!("Starting Held-Karp parallel solver for instance");
    let mut edge_states = Matrix::new(
        vec![EdgeState::Available; distances.data().len()],
        distances.dimension(),
    );

    let scaled_distances = Matrix::new(
        distances
            .data()
            .iter()
            .map(|&d| ScaledDistance::from_distance(d))
            .collect(),
        distances.dimension(),
    );

    let mut node_penalties = initial_penalties(&scaled_distances, distances.dimension());
    let mut fixed_degrees = vec![0u32; distances.dimension()];
    let mut bb_counter = 0;

    let mut initial_upper_bound = Distance(0);
    let mut initial_tour = Vec::with_capacity(distances.dimension());
    for i in 0..distances.dimension() {
        initial_tour.push(UnEdge {
            from: Node(i),
            to: Node((i + 1) % distances.dimension()),
        });
        initial_upper_bound += distances.get_data(Node(i), Node((i + 1) % distances.dimension()));
    }
    let best_tour = Arc::new(Mutex::new(UnTour {
        edges: initial_tour,
        cost: initial_upper_bound,
    }));

    let threads_spawned = Arc::new(Mutex::new(1usize));

    explore_node_new_thread(
        distances,
        &scaled_distances,
        &mut edge_states,
        node_penalties.as_mut_slice(),
        fixed_degrees.as_mut_slice(),
        best_tour.clone(),
        &mut bb_counter,
        None,
        0,
        threads_spawned,
    );

    best_tour.lock().unwrap().clone().into()
}

/// TODO: Adapt documentation
///
/// Depth-first branch-and-bound search exploring nodes recursively.
/// Computes a lower bound at each node using Held-Karp lower bound computation and then branches
/// on an edge from the resulting 1-tree.
///
/// TODO: Summarize arguments in Held-Karp State Struct or Smth
fn explore_node_new_thread(
    distances: &Matrix<Distance>,
    scaled_distances: &Matrix<ScaledDistance>,
    edge_states: &mut Matrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    fixed_degrees: &mut [u32],
    best_tour: Arc<Mutex<UnTour>>,
    bb_counter: &mut usize,
    bb_limit: Option<usize>,
    depth: usize,
    threads_spawned: Arc<Mutex<usize>>,
) {
    // Increment the branch count
    *bb_counter += 1;

    if let Some(limit) = bb_limit {
        if *bb_counter >= limit {
            return;
        }
    }

    let (max_iterations, beta) = if depth == 0 {
        (INITIAL_MAX_ITERATIONS, INITIAL_BETA)
    } else {
        (MAX_ITERATIONS, BETA)
    };

    let one_tree = match held_karp_lower_bound_parallel(
        distances,
        scaled_distances,
        edge_states,
        node_penalties,
        // Possibly pass Arc<Mutex<UnTour>> instead of copying the best tour cost each time
        best_tour.clone(),
        max_iterations,
        beta,
    ) {
        Some(LowerBoundOutput::Tour(tour)) => {
            // Found a new tour, that is, an upper bound
            debug!("Found a new best tour with cost {}", tour.cost.0);
            *best_tour.lock().unwrap() = tour;
            return;
        }
        Some(LowerBoundOutput::LowerBound(lower_bound, one_tree)) => {
            let current_upper_bound = best_tour.lock().unwrap().cost;
            // Check if the lower bound is better than the current best cost
            if lower_bound >= current_upper_bound {
                // Prune this node, as we have already found a better tour than the lower bound
                trace!(
                    "Pruning node with lower bound {} >= upper bound {}",
                    lower_bound.0, current_upper_bound.0
                );
                return;
            } else {
                one_tree
            }
        }
        None => {
            // Infeasible node, prune
            return;
        }
    };

    let Some(branching_edge) =
        edge_to_branch_on(scaled_distances, edge_states, node_penalties, &one_tree)
    else {
        // No edge to branch on, so we prune
        return;
    };

    if *threads_spawned.lock().unwrap() <= 8 {
        if (fixed_degrees[branching_edge.from.0] < 2) && (fixed_degrees[branching_edge.to.0] < 2) {
            let mut threads_spawned_guard = threads_spawned.lock().unwrap();
            *threads_spawned_guard += 1;
            drop(threads_spawned_guard);
            thread::scope(|s| {
                // Explore the branch excluding the edge
                {
                    println!("Spawning thread at depth {}", depth);
                    let mut edge_states_clone = edge_states.clone();
                    let mut node_penalties_clone = node_penalties.to_vec();
                    let mut fixed_degrees_clone = fixed_degrees.to_vec();
                    let best_tour_handle = best_tour.clone();
                    let mut bb_counter_clone = bb_counter.clone();
                    let threads_spawned_handle = threads_spawned.clone();
                    s.spawn(move || {
                        edge_states_clone.set_data_symmetric(
                            branching_edge.from,
                            branching_edge.to,
                            EdgeState::Excluded,
                        );

                        explore_node_new_thread(
                            distances,
                            scaled_distances,
                            &mut edge_states_clone,
                            &mut node_penalties_clone,
                            &mut fixed_degrees_clone,
                            best_tour_handle,
                            &mut bb_counter_clone,
                            bb_limit,
                            depth + 1,
                            threads_spawned_handle,
                        );
                    });
                }

                // Try exploring the branch including the edge.
                // That is, we might not be able to explore this branch, if we the edge inclusion
                // would violate the already fixed degrees / edges.
                println!("Exploring inclusion at depth {}", depth);
                edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Fixed,
                );
                fixed_degrees[branching_edge.from.0] += 1;
                fixed_degrees[branching_edge.to.0] += 1;

                explore_node_new_thread(
                    distances,
                    scaled_distances,
                    edge_states,
                    node_penalties,
                    fixed_degrees,
                    best_tour,
                    bb_counter,
                    bb_limit,
                    depth + 1,
                    threads_spawned.clone(),
                );

                // Backtrack
                edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Available,
                );
                fixed_degrees[branching_edge.from.0] -= 1;
                fixed_degrees[branching_edge.to.0] -= 1;

                // Decrement the thread count
                // *threads_spawned.lock().unwrap() -= 1

                println!("Joining thread at depth {}", depth);
            });
        } else {
            // Explore the branch excluding the edge
            {
                edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Excluded,
                );

                explore_node_new_thread(
                    distances,
                    scaled_distances,
                    edge_states,
                    node_penalties,
                    fixed_degrees,
                    best_tour.clone(),
                    bb_counter,
                    bb_limit,
                    depth + 1,
                    threads_spawned,
                );
            }
        }
    } else {
        println!("Continuing sequentially at depth {}", depth);
        // Explore the branch excluding the edge
        {
            edge_states.set_data_symmetric(
                branching_edge.from,
                branching_edge.to,
                EdgeState::Excluded,
            );

            explore_node_parallel(
                distances,
                scaled_distances,
                edge_states,
                node_penalties,
                fixed_degrees,
                best_tour.clone(),
                bb_counter,
                bb_limit,
                depth + 1,
            );
        }

        // Try exploring the branch including the edge.
        // That is, we might not be able to explore this branch, if we the edge inclusion would
        // violate the already fixed degrees / edges.
        if (fixed_degrees[branching_edge.from.0] < 2) && (fixed_degrees[branching_edge.to.0] < 2) {
            edge_states.set_data_symmetric(
                branching_edge.from,
                branching_edge.to,
                EdgeState::Fixed,
            );
            fixed_degrees[branching_edge.from.0] += 1;
            fixed_degrees[branching_edge.to.0] += 1;

            explore_node_parallel(
                distances,
                scaled_distances,
                edge_states,
                node_penalties,
                fixed_degrees,
                best_tour,
                bb_counter,
                bb_limit,
                depth + 1,
            );

            // Backtrack
            edge_states.set_data_symmetric(
                branching_edge.from,
                branching_edge.to,
                EdgeState::Available,
            );
            fixed_degrees[branching_edge.from.0] -= 1;
            fixed_degrees[branching_edge.to.0] -= 1;
        }
    }
}

/// TODO: Adapt documentation
///
/// Depth-first branch-and-bound search exploring nodes recursively.
/// Computes a lower bound at each node using Held-Karp lower bound computation and then branches
/// on an edge from the resulting 1-tree.
///
/// TODO: Summarize arguments in Held-Karp State Struct or Smth
fn explore_node_parallel(
    distances: &Matrix<Distance>,
    scaled_distances: &Matrix<ScaledDistance>,
    edge_states: &mut Matrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    fixed_degrees: &mut [u32],
    best_tour: Arc<Mutex<UnTour>>,
    bb_counter: &mut usize,
    bb_limit: Option<usize>,
    depth: usize,
) {
    // Increment the branch count
    *bb_counter += 1;

    if let Some(limit) = bb_limit {
        if *bb_counter >= limit {
            return;
        }
    }

    let (max_iterations, beta) = if depth == 0 {
        (INITIAL_MAX_ITERATIONS, INITIAL_BETA)
    } else {
        (MAX_ITERATIONS, BETA)
    };

    let current_upper_bound = best_tour.lock().unwrap().cost;
    let one_tree = match held_karp_lower_bound_parallel(
        distances,
        scaled_distances,
        edge_states,
        node_penalties,
        // Possibly pass Arc<Mutex<UnTour>> instead of copying the best tour cost each time
        best_tour.clone(),
        max_iterations,
        beta,
    ) {
        Some(LowerBoundOutput::Tour(tour)) => {
            // Found a new tour, that is, an upper bound
            debug!("Found a new best tour with cost {}", tour.cost.0);
            *best_tour.lock().unwrap() = tour;
            return;
        }
        Some(LowerBoundOutput::LowerBound(lower_bound, one_tree)) => {
            // Check if the lower bound is better than the current best cost
            if lower_bound >= current_upper_bound {
                // Prune this node, as we have already found a better tour than the lower bound
                trace!(
                    "Pruning node with lower bound {} >= upper bound {}",
                    lower_bound.0, current_upper_bound.0
                );
                return;
            } else {
                one_tree
            }
        }
        None => {
            // Infeasible node, prune
            return;
        }
    };

    let Some(branching_edge) =
        edge_to_branch_on(scaled_distances, edge_states, node_penalties, &one_tree)
    else {
        // No edge to branch on, so we prune
        return;
    };

    // Explore the branch excluding the edge
    {
        edge_states.set_data_symmetric(branching_edge.from, branching_edge.to, EdgeState::Excluded);

        explore_node_parallel(
            distances,
            scaled_distances,
            edge_states,
            node_penalties,
            fixed_degrees,
            best_tour.clone(),
            bb_counter,
            bb_limit,
            depth + 1,
        );

        edge_states.set_data_symmetric(
            branching_edge.from,
            branching_edge.to,
            EdgeState::Available,
        );
    }

    // Try exploring the branch including the edge.
    // That is, we might not be able to explore this branch, if we the edge inclusion would violate
    // the already fixed degrees / edges.
    if (fixed_degrees[branching_edge.from.0] < 2) && (fixed_degrees[branching_edge.to.0] < 2) {
        edge_states.set_data_symmetric(branching_edge.from, branching_edge.to, EdgeState::Fixed);
        fixed_degrees[branching_edge.from.0] += 1;
        fixed_degrees[branching_edge.to.0] += 1;

        explore_node_parallel(
            distances,
            scaled_distances,
            edge_states,
            node_penalties,
            fixed_degrees,
            best_tour,
            bb_counter,
            bb_limit,
            depth + 1,
        );

        // Backtrack
        edge_states.set_data_symmetric(
            branching_edge.from,
            branching_edge.to,
            EdgeState::Available,
        );
        fixed_degrees[branching_edge.from.0] -= 1;
        fixed_degrees[branching_edge.to.0] -= 1;
    }
}

enum LowerBoundOutput {
    LowerBound(Distance, Vec<UnEdge>),
    Tour(UnTour),
}

/// Compute Held-Karp lower bound using 1-trees and Lagrangian relaxation
fn held_karp_lower_bound_parallel(
    distances: &Matrix<Distance>,
    scaled_distances: &Matrix<ScaledDistance>,
    edge_states: &Matrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    best_tour: Arc<Mutex<UnTour>>,
    max_iterations: usize,
    beta: f64,
) -> Option<LowerBoundOutput> {
    // Tracks the current best lower bound found
    let mut scaled_best_lower_bound = ScaledDistance::MIN;

    let mut iter_count = 0;

    let mut alpha = INITIAL_ALPHA;

    let node_penalty_sum: ScaledDistance = node_penalties.iter().sum();

    let one_tree = loop {
        let one_tree = min_one_tree(scaled_distances, edge_states, node_penalties)?;

        let scaled_upper_bound = ScaledDistance::from_distance(best_tour.lock().unwrap().cost);

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
        if one_tree_cost >= scaled_upper_bound {
            // Lower bound exceeds current upper bound, prune
            trace!(
                "Pruning in held_karp_lower_bound due to lower bound {} >= upper bound {}",
                one_tree_cost.0, scaled_upper_bound.0
            );
            break one_tree;
        }

        // Next we check the degrees of the nodes in the 1-tree
        // Deg[node] can be interpreted as follows:
        //  Deg[node] < 0: Node has degree > 2 -> we need to decrease its penalty. This makes edges
        //                 incident to node more expensive, that is, less likely to be selected.
        //  Deg[node] > 0: Node has degree < 2 -> we need to increase its penalty. This makes edges
        //                 incident to node cheaper, that is, more likely to be selected.
        //  Deg[node] == 0: Node has degree == 2 -> no change to penalty.
        let mut deg = vec![2i32; distances.dimension()];

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
            break one_tree;
        }

        // TODO: Research on subgradient method for non-smooth optimization to find out more about
        // this
        let step_size = (alpha
            * ((scaled_upper_bound.0 - one_tree_cost.0) as f64 / (square_sum as f64)))
            as i32;

        if step_size <= 3 {
            // Step size is very small (<= 3 in scaled), we probably won't be making much progress
            break one_tree;
        }

        alpha *= beta;

        // Update penalties based on degree deviations and step size
        // TODO: Handle overflows
        for (node_penalty, &d) in node_penalties.iter_mut().zip(deg.iter()) {
            let adjustment = ScaledDistance(step_size * d);
            *node_penalty += adjustment;
        }
    };

    let best_lower_bound = scaled_best_lower_bound.to_distance_rounded_up();

    Some(LowerBoundOutput::LowerBound(best_lower_bound, one_tree))
}
