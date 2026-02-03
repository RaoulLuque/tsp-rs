use std::{
    sync::{Arc, Mutex},
    thread,
};

use log::{debug, info, trace};
use tsp_core::instance::{
    UnTour,
    distance::{Distance, ScaledDistance},
    edge::UnEdge,
    matrix::SquareMatrix,
    node::Node,
};

use crate::held_karp_mod::{
    EdgeState, HeldKarpState, LowerBoundOutput, UpperBoundProvider, edge_to_branch_on,
    held_karp_lower_bound, initial_penalties,
};

const INITIAL_MAX_ITERATIONS: usize = 1_000;
// TODO: Possibly increase this even further. Possible downside: Longer runtimes on easy instances
// where we don't branch. Possible upside: We find better tours on hard instances as the threads
// have more time to improve the lower bound before we branch.
const MAX_ITERATIONS: usize = 500;

const INITIAL_BETA: f64 = 0.99;
const BETA: f64 = 0.9;

/// Solve the Traveling Salesperson Problem using a parallel implementation of the Held-Karp
/// algorithm.
///
/// For a detailed explanation of the algorithm, see the [module-level
/// documentation][crate::held_karp_mod].
/// This implementation is the same as the sequential one
/// [`held_karp`][crate::held_karp_mod::held_karp], except that when branching, if both branches can
/// be explored, one branch is explored in a new thread (if a core is available) while the other
/// branch is explored in the current thread.
///
/// This should speed up the solving process on multi-core systems, especially for hard instances
/// where a lot of branching is required and possibly even lead to better tours being found.
pub fn held_karp_parallel(distances: &SquareMatrix<Distance>) -> UnTour {
    info!("Starting Held-Karp parallel solver");
    let edge_states = SquareMatrix::new(
        vec![EdgeState::Available; distances.data().len()],
        distances.dimension(),
    );

    let scaled_distances = SquareMatrix::new(
        distances
            .data()
            .iter()
            .map(|&d| ScaledDistance::from_distance(d))
            .collect(),
        distances.dimension(),
    );

    let node_penalties = initial_penalties(&scaled_distances, distances.dimension());
    let fixed_degrees = vec![0u32; distances.dimension()];
    let bb_counter = 0;

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

    let mut state = HeldKarpState {
        edge_states,
        node_penalties,
        fixed_degrees,
        best_tour: best_tour.clone(),
        bb_counter,
        depth: 0,
    };

    explore_node_parallel(
        distances,
        &scaled_distances,
        &mut state,
        None,
        threads_spawned,
    );

    best_tour.clone().lock().unwrap().clone()
}

/// Same as [`explore_node`][crate::held_karp_mod::explore_node] but parallelized.
///
/// That is, if both branches can be explored, one branch is explored in a new thread (if a core is
/// available) while the other branch is explored in the current thread.
fn explore_node_parallel(
    distances: &SquareMatrix<Distance>,
    scaled_distances: &SquareMatrix<ScaledDistance>,
    state: &mut HeldKarpState<Arc<Mutex<UnTour>>>,
    bb_limit: Option<usize>,
    threads_spawned: Arc<Mutex<usize>>,
) {
    // Increment the branch count
    state.bb_counter += 1;

    if let Some(limit) = bb_limit {
        if state.bb_counter >= limit {
            return;
        }
    }

    let (max_iterations, beta) = if state.depth == 0 {
        (INITIAL_MAX_ITERATIONS, INITIAL_BETA)
    } else {
        (MAX_ITERATIONS, BETA)
    };

    let one_tree = match held_karp_lower_bound(
        distances,
        scaled_distances,
        &state.edge_states,
        &mut state.node_penalties,
        // Possibly pass Arc<Mutex<UnTour>> instead of copying the best tour cost each time
        state.best_tour.clone(),
        max_iterations,
        beta,
    ) {
        Some(LowerBoundOutput::Tour(tour)) => {
            // Found a new tour, that is, an upper bound
            debug!("Found a new best tour with cost {}", tour.cost.0);
            *state.best_tour.lock().unwrap() = tour;
            return;
        }
        Some(LowerBoundOutput::LowerBound(lower_bound, one_tree)) => {
            let current_upper_bound = state.best_tour.lock().unwrap().cost;
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

    let Some(branching_edge) = edge_to_branch_on(
        scaled_distances,
        &state.edge_states,
        &state.node_penalties,
        &one_tree,
    ) else {
        // No edge to branch on, so we prune
        return;
    };

    state.depth += 1;

    // We distinguish the following cases:
    // 1. We can explore both branches (including and excluding the edge) - outermost if
    //
    //      1.true  We check, whether we can spawn a new thread (i.e., whether a core is available)
    //
    //      2.false Since we only explore one branch, we explore it in the current thread
    if (state.fixed_degrees[branching_edge.from.0] < 2)
        && (state.fixed_degrees[branching_edge.to.0] < 2)
    {
        if *threads_spawned.lock().unwrap() <= 8 {
            // We can spawn a new thread which explores the branch excluding the edge
            *threads_spawned.lock().unwrap() += 1;
            thread::scope(|s| {
                // Explore the branch excluding the edge
                let _ = {
                    let mut state_cloned = state.clone_custom();
                    let threads_spawned_handle = threads_spawned.clone();

                    let thread_handle = s.spawn(move || {
                        state_cloned.edge_states.set_data_symmetric(
                            branching_edge.from,
                            branching_edge.to,
                            EdgeState::Excluded,
                        );

                        explore_node_parallel(
                            distances,
                            scaled_distances,
                            &mut state_cloned,
                            bb_limit,
                            threads_spawned_handle,
                        );
                    });

                    thread_handle
                };

                // Try exploring the branch including the edge.
                // That is, we might not be able to explore this branch, if we the edge inclusion
                // would violate the already fixed degrees / edges.
                state.edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Fixed,
                );
                state.fixed_degrees[branching_edge.from.0] += 1;
                state.fixed_degrees[branching_edge.to.0] += 1;

                explore_node_parallel(
                    distances,
                    scaled_distances,
                    state,
                    bb_limit,
                    threads_spawned.clone(),
                );

                // Backtrack
                state.edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Available,
                );
                state.fixed_degrees[branching_edge.from.0] -= 1;
                state.fixed_degrees[branching_edge.to.0] -= 1;
            });

            // Decrement the thread count
            *threads_spawned.lock().unwrap() -= 1;
        } else {
            // We cannot spawn a new thread, so we explore both branches in the current thread
            {
                state.edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Excluded,
                );

                explore_node_parallel(
                    distances,
                    scaled_distances,
                    state,
                    bb_limit,
                    threads_spawned.clone(),
                );
            }

            // Try exploring the branch including the edge.
            // That is, we might not be able to explore this branch, if we the edge inclusion would
            // violate the already fixed degrees / edges.
            if (state.fixed_degrees[branching_edge.from.0] < 2)
                && (state.fixed_degrees[branching_edge.to.0] < 2)
            {
                state.edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Fixed,
                );
                state.fixed_degrees[branching_edge.from.0] += 1;
                state.fixed_degrees[branching_edge.to.0] += 1;

                explore_node_parallel(
                    distances,
                    scaled_distances,
                    state,
                    bb_limit,
                    threads_spawned.clone(),
                );

                // Backtrack
                state.edge_states.set_data_symmetric(
                    branching_edge.from,
                    branching_edge.to,
                    EdgeState::Available,
                );
                state.fixed_degrees[branching_edge.from.0] -= 1;
                state.fixed_degrees[branching_edge.to.0] -= 1;
            }
        }
    } else {
        // We can only explore the branch excluding the edge.
        {
            state.edge_states.set_data_symmetric(
                branching_edge.from,
                branching_edge.to,
                EdgeState::Excluded,
            );

            explore_node_parallel(
                distances,
                scaled_distances,
                state,
                bb_limit,
                threads_spawned,
            );
        }
    }
}

impl HeldKarpState<Arc<Mutex<UnTour>>> {
    /// Clone the HeldKarpState, creating new copies of the internal data structures, except for
    /// the best_tour which is shared.
    fn clone_custom(&self) -> Self {
        HeldKarpState {
            edge_states: self.edge_states.clone(),
            node_penalties: self.node_penalties.to_vec(),
            fixed_degrees: self.fixed_degrees.to_vec(),
            best_tour: self.best_tour.clone(),
            bb_counter: self.bb_counter,
            depth: self.depth,
        }
    }
}

impl UpperBoundProvider for Arc<Mutex<UnTour>> {
    fn get_upper_bound(&self) -> Distance {
        self.lock().unwrap().cost
    }
}
