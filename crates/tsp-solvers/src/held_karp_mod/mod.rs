/*!
This module contains an implementation of the
[Held-Karp algorithm](https://en.wikipedia.org/wiki/Held%E2%80%93Karp_algorithm)
(also known as the Bellman-Held-Karp algorithm) for solving the Traveling Salesperson Problem.

## Top-level Description of the Algorithm

The algorithm uses branch-and-bound and Lagrangian relaxation to successively
tighten lower and upper bounds simultaneously until the bounds converge to an optimal solution.

The branch-and-bound part of the algorithm systematically explores the space of possible tours by
branching on edges (including or excluding them from the tour) and pruning branches that cannot
yield a better solution than the best one found so far.

For finding lower bounds, the algorithm uses [1-trees](#1-trees), which are minimum spanning trees that span all nodes
except one, plus two edges connecting the excluded node to the tree. The cost of a minimum 1-tree
is in this case a lower bound on the cost of a TSP tour. By introducing node penalties and adjusting
them based on the degrees of nodes in the 1-tree, we can iteratively improve the lower bound and
nudge the 1-tree towards a valid TSP tour (the process of adjusting penalties is a form of
[Lagrangian relaxation](#lagrangian-relaxation).

We get upper bounds (that is, valid tours) via our 1-trees. When a 1-tree happens to be a valid tour
(that is, all nodes have degree 2), we have found a (possible) new upper bound. We keep track of the best
upper bound found so far and use it to prune branches in the branch-and-bound search.

## Call Structure of the Algorithm

The call structure of the algorithm and sub-methods is as follows. Indented functions indicate
that they are called by the function above them.
- `held_karp`:  Main entry point for the Held-Karp solver. Sets up parameters and initiates the
                branch-and-bound search.
    - `explore_node`:   Performs depth-first branch-and-bound search.
        - `held_karp_lower_bound`:  Computes a lower bound using 1-trees and Lagrangian relaxation.
            - `min_one_tree`:   Computes a minimum 1-tree given current edge states and node penalties.
                - `min_spanning_tree`:  Computes a minimum spanning tree of all nodes except the
                                        first using Prim's algorithm.
        - `edge_to_branch_on`:  Selects an edge (from the 1-tree) to branch on.
        - `explore_node`:   Is called twice (recursively) to explore the branches including or excluding
                            the selected edge.

## 1-trees

1-trees are minimum spanning trees that span nodes 2 to n, plus two minimum cost edges
connecting node 1 to the tree. This is always a lower bound on the cost of a TSP tour, since
any TSP tour is a 1-tree. To see the latter, take any valid TSP tour, remove the edges adjacent to
the first node, and one obtains a spanning tree. Thus, the cheapest 1-tree provides a lower bound
on the TSP tour cost.

## Lagrangian Relaxation

Because 1-trees by themselves might have many nodes with degree unequal to 2 (and thus are 'far
away' from being a valid TSP tour), we introduce node penalties that adjust the costs of edges
incident to each node. By iteratively adjusting the penalties based on the degree of nodes in the 1-tree,
we can converge towards 1-tree closer to an actual valid tour and thus a tighter lower bound on the
TSP tour cost.

Once an actual tour is found, we can use that as an upper bound to prune the search space in the branch-and-bound
exploration.

This is considered a lagrangian relaxation ([wikipedia](https://en.wikipedia.org/wiki/Lagrangian_relaxation))
since instead of enforcing the degree-2 constraints strictly for our 1-trees, we instead penalize
deviations from degree 2 via the node penalties.

## Edge States

Edges can be in one of three states: Available, Excluded, or Fixed. This allows the
branch-and-bound search to systematically explore different configurations of the TSP tour
by forcibly including or excluding edges.

## References and Credit

- [Concorde TSP Solver](https://www.math.uwaterloo.ca/tsp/concorde.html): The Concorde TSP solver
  is a well-known implementation of TSP algorithms, including the Held-Karp algorithm.
  The implementation in this module is highly inspired by Concorde and its implementation.
- [The Traveling Salesman Problem: A Computational Study](https://www.degruyterbrill.com/document/doi/10.1515/9781400841103/html?lang=en)
  by David L. Applegate, Robert E. Bixby, Vasek Chvatal, and William J. Cook.
  This book provides an in-depth treatment of various TSP algorithms, including the Held-Karp algorithm.

*/

use std::u32;

use log::{debug, info, trace};
use tsp_core::instance::{
    UnTour,
    distance::{Distance, ScaledDistance},
    edge::{UnEdge, data::EdgeDataMatrix},
    node::Node,
};

use crate::held_karp_mod::trees::min_one_tree;

mod trees;

/// Solve the Traveling Salesman Problem using the Held-Karp algorithm.
///
/// For a detailed explanation of the algorithm, see the [module-level
/// documentation][crate::held_karp_mod].
pub fn held_karp(distances: &EdgeDataMatrix<Distance>) -> Option<UnTour> {
    info!("Starting Held-Karp solver for instance");
    let mut edge_states = EdgeDataMatrix {
        data: vec![EdgeState::Available; distances.data.len()],
        dimension: distances.dimension,
    };

    let scaled_distances = EdgeDataMatrix {
        data: distances
            .data
            .iter()
            .map(|&d| ScaledDistance::from_distance(d))
            .collect(),
        dimension: distances.dimension,
    };

    let mut node_penalties = initial_penalties(&scaled_distances, distances.dimension);
    let mut fixed_degrees = vec![0u32; distances.dimension];
    let mut bb_counter = 0;

    let mut initial_upper_bound = Distance(0);
    let mut initial_tour = Vec::with_capacity(distances.dimension);
    for i in 0..distances.dimension {
        initial_tour.push(UnEdge {
            from: Node(i),
            to: Node((i + 1) % distances.dimension),
        });
        initial_upper_bound += distances.get_data(Node(i), Node((i + 1) % distances.dimension));
    }
    let mut best_tour = Some(UnTour {
        edges: initial_tour,
        cost: initial_upper_bound,
    });

    explore_node(
        distances,
        &scaled_distances,
        &mut edge_states,
        node_penalties.as_mut_slice(),
        fixed_degrees.as_mut_slice(),
        &mut initial_upper_bound,
        &mut best_tour,
        &mut bb_counter,
        None,
        0,
    );

    best_tour
}

const INITIAL_MAX_ITERATIONS: usize = 1_000;
const MAX_ITERATIONS: usize = 10;

const INITIAL_ALPHA: f64 = 2.0;

const INITIAL_BETA: f64 = 0.99;
const BETA: f64 = 0.9;

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State of an edge in the branch-and-bound search.
enum EdgeState {
    /// Edge is available for inclusion or exclusion, i.e. not yet decided.
    Available = 1,
    /// Edge is currently excluded from the tour and thus 1-trees / spanning trees.
    Excluded = 0,
    /// Edge is currently fixed to be included in the tour and thus 1-trees / spanning trees.
    Fixed = -1,
}

/// Depth-first branch-and-bound search exploring nodes recursively.
/// Computes a lower bound at each node using Held-Karp lower bound computation and then branches
/// on an edge from the resulting 1-tree.
///
/// TODO: Summarize arguments in Held-Karp State Struct or Smth
/// TODO: Possibly remove upper_bound as best_tour.cost already contains that information
fn explore_node(
    distances: &EdgeDataMatrix<Distance>,
    scaled_distances: &EdgeDataMatrix<ScaledDistance>,
    edge_states: &mut EdgeDataMatrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    fixed_degrees: &mut [u32],
    upper_bound: &mut Distance,
    best_tour: &mut Option<UnTour>,
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

    let one_tree = match held_karp_lower_bound(
        distances,
        scaled_distances,
        edge_states,
        node_penalties,
        *upper_bound,
        max_iterations,
        beta,
    ) {
        Some(LowerBoundOutput::Tour(tour)) => {
            // Found a new tour, that is, an upper bound
            info!("Found a new best tour with cost {}", tour.cost.0);
            *upper_bound = tour.cost;
            *best_tour = Some(tour);
            return;
        }
        Some(LowerBoundOutput::LowerBound(lower_bound, one_tree)) => {
            // Check if the lower bound is better than the current best cost
            if lower_bound >= *upper_bound {
                // Prune this node, as we have already found a better tour than the lower bound
                debug!(
                    "Pruning node with lower bound {} >= upper bound {}",
                    lower_bound.0, upper_bound.0
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
        edge_states.set_data(branching_edge.from, branching_edge.to, EdgeState::Excluded);

        explore_node(
            distances,
            scaled_distances,
            edge_states,
            node_penalties,
            fixed_degrees,
            upper_bound,
            best_tour,
            bb_counter,
            bb_limit,
            depth + 1,
        );

        edge_states.set_data(branching_edge.from, branching_edge.to, EdgeState::Available);
    }

    // Try exploring the branch including the edge.
    // That is, we might not be able to explore this branch, if we the edge inclusion would violate
    // the already fixed degrees / edges.
    if (fixed_degrees[branching_edge.from.0] < 2) && (fixed_degrees[branching_edge.to.0] < 2) {
        edge_states.set_data(branching_edge.from, branching_edge.to, EdgeState::Fixed);
        fixed_degrees[branching_edge.from.0] += 1;
        fixed_degrees[branching_edge.to.0] += 1;

        explore_node(
            distances,
            scaled_distances,
            edge_states,
            node_penalties,
            fixed_degrees,
            upper_bound,
            best_tour,
            bb_counter,
            bb_limit,
            depth + 1,
        );

        // Backtrack
        edge_states.set_data(branching_edge.from, branching_edge.to, EdgeState::Available);
        fixed_degrees[branching_edge.from.0] -= 1;
        fixed_degrees[branching_edge.to.0] -= 1;
    }
}

enum LowerBoundOutput {
    LowerBound(Distance, Vec<UnEdge>),
    Tour(UnTour),
}

/// Compute Held-Karp lower bound using 1-trees and Lagrangian relaxation
fn held_karp_lower_bound(
    distances: &EdgeDataMatrix<Distance>,
    scaled_distances: &EdgeDataMatrix<ScaledDistance>,
    edge_states: &EdgeDataMatrix<EdgeState>,
    node_penalties: &mut [ScaledDistance],
    upper_bound: Distance,
    max_iterations: usize,
    beta: f64,
) -> Option<LowerBoundOutput> {
    let scaled_upper_bound = ScaledDistance::from_distance(upper_bound);

    // Tracks the current best lower bound found
    let mut scaled_best_lower_bound = ScaledDistance::MIN;

    let mut iter_count = 0;

    let mut alpha = INITIAL_ALPHA;

    let node_penalty_sum: ScaledDistance = node_penalties.iter().sum();

    let one_tree = loop {
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

/// Select an edge from the 1-tree to branch on.
///
/// The edge with the minimum reduced cost (edge_cost - node_penalties[from] - node_penalties[to])
/// among available edges is selected for branching.
fn edge_to_branch_on(
    scaled_distances: &EdgeDataMatrix<ScaledDistance>,
    edge_states: &EdgeDataMatrix<EdgeState>,
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
    scaled_distances: &EdgeDataMatrix<ScaledDistance>,
    dimension: usize,
) -> Vec<ScaledDistance> {
    let mut penalties = vec![ScaledDistance::MAX; dimension];

    for from in 0..dimension {
        for to in 0..from {
            let distance = scaled_distances.get_data_from_seq(Node(from), Node(to));
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
