/*!
This module contains an implementation of the
[Held-Karp algorithm](https://en.wikipedia.org/wiki/Held%E2%80%93Karp_algorithm)
(also known as the Bellman-Held-Karp algorithm) for solving the Traveling Salesperson Problem. It is
based on the implementation in the [Concorde TSP Solver](https://www.math.uwaterloo.ca/tsp/concorde.html).

## Top-level Description of the Algorithm

The algorithm uses branch-and-bound and Lagrangian relaxation to successively
approach a feasible solution.

The branch-and-bound part of the algorithm systematically explores the space of possible tours by
branching on edges (including or excluding them from the tour) and pruning branches that cannot
yield a better solution than the best one found so far.

For finding lower bounds, the algorithm uses [1-trees](#1-trees), which are minimum spanning trees that span all nodes
except one, plus two edges connecting the excluded node to the tree. The cost of the spanning tree
is in this case a lower bound on the cost of a TSP tour. By introducing node penalties and adjusting
them based on the degrees of nodes in the 1-tree, we can iteratively
nudge the 1-tree towards a valid TSP tour (the process of adjusting penalties is a form of
[Lagrangian relaxation](#lagrangian-relaxation), however loosing the property of being a lower bound.
We thus refer to the computed values as 'pseudo-lower bounds').

We get upper bounds (that is, valid tours) via our 1-trees. When a 1-tree happens to be a valid tour
(that is, all nodes have degree 2), we have found a (possible) new upper bound. We keep track of the best
upper bound found so far and use it to prune branches in the branch-and-bound search.

## Call Structure of the Algorithm

The call structure of the algorithm and sub-methods is as follows. Indented functions indicate
that they are called by the function above them.
- `held_karp`:  Main entry point for the Held-Karp solver. Sets up parameters and initiates the
                branch-and-bound search.
    - `explore_node`:   Performs depth-first branch-and-bound search.
        - `held_karp_lower_bound`:  Computes a pseudo-lower bound using 1-trees and Lagrangian relaxation.
            - `min_one_tree`:   Computes a minimum 1-tree given current edge states and node penalties.
                - `min_spanning_tree`:  Computes a minimum spanning tree of all nodes except the
                                        first using Prim's algorithm.
        - `edge_to_branch_on`:  Selects an edge (from the 1-tree) to branch on.
        - `explore_node`:   Is called twice (recursively) to explore the branches including or excluding
                            the selected edge.

## 1-trees

1-trees are minimum spanning trees that span nodes 2 to n, plus two minimum cost edges
connecting node 1 to the tree. The cheapest 1-tree provides a lower bound for the cost of any
TSP tour, since any TSP tour is a 1-tree. To see the latter, take any valid TSP tour, remove the
edges adjacent to the first node, and one obtains a spanning tree. However, as this algorithm
works with penalties, the 1-trees found might not be the cheapest with respect to the original distances,
and thus are not technically lower bounds.

## Lagrangian Relaxation

Because 1-trees by themselves might have many nodes with degree unequal to 2 (and thus are 'far
away' from being a valid TSP tour), we introduce node penalties that adjust the costs of edges
incident to each node. By iteratively adjusting the penalties based on the degree of nodes in the 1-tree,
we can converge towards 1-trees closer to an actual valid tour.

Once an actual tour is found, we can use that as an upper bound to prune the search space in the branch-and-bound
exploration.

This is considered a Lagrangian relaxation ([wikipedia](https://en.wikipedia.org/wiki/Lagrangian_relaxation))
since instead of enforcing the degree-2 constraints strictly for our 1-trees, we instead penalize
deviations from degree 2 via the node penalties.

## Edge States

Edges can be in one of three states: Available, Excluded, or Fixed. This allows the
branch-and-bound search to systematically explore different configurations of the TSP tour
by forcibly including or excluding edges.

## References and Credit

- [Concorde]: The Concorde TSP solver
  is a well-known implementation of TSP algorithms, including the Held-Karp algorithm.
  The implementation in this module took inspiration from techniques used in Concorde's approach to the Held-Karp algorithm.
  In particular, the [min_spanning_tree](trees::min_spanning_tree) implementation of Prim's algorithm
  is adapted from Concorde's implementation, which was written by Sanjeeb Dash.
- [The Traveling Salesman Problem: A Computational Study](https://www.degruyterbrill.com/document/doi/10.1515/9781400841103/html?lang=en)
  by David L. Applegate, Robert E. Bixby, Vasek Chvatal, and William J. Cook.
  This book provides an in-depth treatment of various TSP algorithms, including the Held-Karp algorithm.

[Concorde]: https://www.math.uwaterloo.ca/tsp/concorde.html
*/

use std::u32;

use log::{debug, info, trace};
use tsp_core::instance::{
    UnTour,
    distance::{Distance, ScaledDistance},
    edge::UnEdge,
    matrix::SquareMatrix,
    node::Node,
};

pub use crate::held_karp_mod::{parallel::held_karp_parallel, trees::min_one_tree};

mod parallel;
mod trees;

// TODO: Tune this maximum penalty value
const WEIGHT_MAX_NODE: ScaledDistance = ScaledDistance(1 << 21);

/// Struct to track the state of the Held-Karp algorithm during branch-and-bound search.
///
/// Only includes variables that are mutated during the search.
struct HeldKarpState<Tour> {
    edge_states: SquareMatrix<EdgeState>,
    node_penalties: Vec<ScaledDistance>,
    fixed_degrees: Vec<u32>,
    best_tour: Tour,
    bb_counter: usize,
    depth: usize,
}

/// Solve the Traveling Salesman Problem using the Held-Karp algorithm.
///
/// For a detailed explanation of the algorithm, see the [module-level
/// documentation][crate::held_karp_mod].
pub fn held_karp(distances: &SquareMatrix<Distance>) -> UnTour {
    info!("Starting Held-Karp solver");
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
    let best_tour = UnTour {
        edges: initial_tour,
        cost: initial_upper_bound,
    };

    let mut held_karp_state = HeldKarpState {
        edge_states,
        node_penalties,
        fixed_degrees,
        best_tour,
        bb_counter,
        depth: 0,
    };

    explore_node(distances, &scaled_distances, None, &mut held_karp_state);

    held_karp_state.best_tour
}

const INITIAL_MAX_ITERATIONS: usize = 1_000;
const MAX_ITERATIONS: usize = 10;

const INITIAL_ALPHA: f64 = 2.0;

const INITIAL_BETA: f64 = 0.99;
const BETA: f64 = 0.9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State of an edge in the branch-and-bound search.
pub enum EdgeState {
    /// Edge is available for inclusion or exclusion, i.e. not yet decided.
    Available = 1,
    /// Edge is currently excluded from the tour and thus 1-trees / spanning trees.
    Excluded = 0,
    /// Edge is currently fixed to be included in the tour and thus 1-trees / spanning trees.
    Fixed = -1,
}

/// Depth-first branch-and-bound search exploring nodes recursively.
/// Computes a pseudo-lower bound at each node using Held-Karp lower bound computation and then
/// branches on an edge from the resulting 1-tree.
fn explore_node(
    distances: &SquareMatrix<Distance>,
    scaled_distances: &SquareMatrix<ScaledDistance>,
    bb_limit: Option<usize>,
    state: &mut HeldKarpState<UnTour>,
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
        state.node_penalties.as_mut_slice(),
        state.best_tour.cost,
        max_iterations,
        beta,
    ) {
        Some(LowerBoundOutput::Tour(tour)) => {
            // Found a new tour, that is, an upper bound
            debug!("Found a new best tour with cost {}", tour.cost.0);
            state.best_tour = tour;
            return;
        }
        Some(LowerBoundOutput::LowerBound(lower_bound, one_tree)) => {
            // Check if the lower bound is better than the current best cost
            if lower_bound >= state.best_tour.cost {
                // Prune this node, as we have already found a better tour than the lower bound
                trace!(
                    "Pruning node with lower bound {} >= upper bound {}",
                    lower_bound.0, state.best_tour.cost.0
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

    // Explore the branch excluding the edge
    {
        trace!("Branching on edge {:?} by excluding it", branching_edge);
        state.edge_states.set_data_symmetric(
            branching_edge.from,
            branching_edge.to,
            EdgeState::Excluded,
        );

        explore_node(distances, scaled_distances, bb_limit, state);

        state.edge_states.set_data_symmetric(
            branching_edge.from,
            branching_edge.to,
            EdgeState::Available,
        );
    }

    // Try exploring the branch including the edge.
    // That is, we might not be able to explore this branch, if we the edge inclusion would violate
    // the already fixed degrees / edges.
    if (state.fixed_degrees[branching_edge.from.0] < 2)
        && (state.fixed_degrees[branching_edge.to.0] < 2)
    {
        trace!("Branching on edge {:?} by including it", branching_edge);

        state.edge_states.set_data_symmetric(
            branching_edge.from,
            branching_edge.to,
            EdgeState::Fixed,
        );
        state.fixed_degrees[branching_edge.from.0] += 1;
        state.fixed_degrees[branching_edge.to.0] += 1;

        explore_node(distances, scaled_distances, bb_limit, state);

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

enum LowerBoundOutput {
    LowerBound(Distance, Vec<UnEdge>),
    Tour(UnTour),
}

pub(crate) trait UpperBoundProvider {
    fn get_upper_bound(&self) -> Distance;
}

impl UpperBoundProvider for Distance {
    fn get_upper_bound(&self) -> Distance {
        *self
    }
}

/// Compute Held-Karp pseudo-lower bound using 1-trees and Lagrangian relaxation
fn held_karp_lower_bound(
    distances: &SquareMatrix<Distance>,
    scaled_distances: &SquareMatrix<ScaledDistance>,
    edge_states: &SquareMatrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    upper_bound_provider: impl UpperBoundProvider,
    max_iterations: usize,
    beta: f64,
) -> Option<LowerBoundOutput> {
    // Tracks the current best lower bound found
    let mut scaled_best_lower_bound = ScaledDistance::MIN;

    let mut iter_count = 0;

    let mut alpha = INITIAL_ALPHA;

    let mut node_penalty_sum: ScaledDistance = node_penalties.iter().sum();

    let one_tree = loop {
        // We only create the variable here, to be able to reuse this function in parallel settings
        // where a new upper_bound might be found between runs of this loop.
        let scaled_upper_bound =
            ScaledDistance::from_distance(upper_bound_provider.get_upper_bound());

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

        if one_tree_cost >= scaled_upper_bound {
            // Lower bound exceeds current upper bound, prune
            trace!(
                "Pruning in held_karp_lower_bound due to lower bound {} > upper bound {}",
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
        let mut overflow = false;

        // Skip node 0, as its degree is always 2 in a 1-tree
        let node_penalties_deg_iter = node_penalties.iter_mut().zip(deg.iter()).skip(1);
        for (node_penalty, &d) in node_penalties_deg_iter {
            let adjustment = ScaledDistance(step_size * d);
            *node_penalty += adjustment;
            if *node_penalty > WEIGHT_MAX_NODE {
                *node_penalty = WEIGHT_MAX_NODE;
                overflow = true;
            }
        }
        if overflow {
            node_penalty_sum = node_penalties.iter().sum();
        }
    };

    let best_lower_bound = scaled_best_lower_bound.to_distance_rounded_up();

    Some(LowerBoundOutput::LowerBound(best_lower_bound, one_tree))
}

/// Select an edge from the 1-tree to branch on.
///
/// The edge with the minimum reduced cost (edge_cost - node_penalties[from] - node_penalties[to])
/// among available edges is selected for branching.
fn edge_to_branch_on(
    scaled_distances: &SquareMatrix<ScaledDistance>,
    edge_states: &SquareMatrix<EdgeState>,
    node_penalties: &[ScaledDistance],
    one_tree: &[UnEdge],
) -> Option<UnEdge> {
    let mut minimum_edge = None;
    let mut minimum_edge_distance = ScaledDistance::MAX;

    for edge in one_tree {
        if edge_states.get_data(edge.from, edge.to) == EdgeState::Available {
            let reduced_distance = scaled_distances.get_data(edge.from, edge.to)
                - node_penalties[edge.from.0]
                - node_penalties[edge.to.0];
            if reduced_distance < minimum_edge_distance {
                minimum_edge_distance = reduced_distance;
                minimum_edge = Some(*edge);
            }
        }
    }

    minimum_edge
}

/// Initializes node penalties for Lagrangian relaxation.
///
/// Node penalties are set to half the minimum distances to other nodes.
fn initial_penalties(
    scaled_distances: &SquareMatrix<ScaledDistance>,
    dimension: usize,
) -> Vec<ScaledDistance> {
    let mut penalties = vec![ScaledDistance::MAX; dimension];

    for from in 0..dimension {
        for to in 0..from {
            let distance = scaled_distances.get_data_to_seq(Node(from), Node(to));
            if distance < penalties[from] {
                penalties[from] = distance;
            }
            if distance < penalties[to] {
                penalties[to] = distance;
            }
        }
    }

    for penalty in penalties.iter_mut() {
        *penalty = *penalty / 2;
    }

    penalties
}
