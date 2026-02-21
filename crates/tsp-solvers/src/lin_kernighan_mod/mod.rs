use std::collections::VecDeque;

use current_tour::CurrentTour;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use tsp_core::instance::{distance::Distance, matrix::SquareMatrix, node::Node};

mod current_tour;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeState {
    Added,
    Deleted,
    None,
}

fn lin_kernighan_loop(distances: &SquareMatrix<Distance>) {
    // TODO: Handle possibly don't provide seed and use random seed instead.
    let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(0);

    let mut edge_state = SquareMatrix::new(
        vec![EdgeState::None; distances.data().len()],
        distances.dimension(),
    );
    let mut current_tour = initialize_current_tour(distances.dimension());

    // Setup initial node queue using a random tour.
    let random_tour = find_random_tour(distances.dimension(), &mut rng);
    let mut node_queue = VecDeque::from(random_tour);

    lin_kernighan(
        distances,
        &mut edge_state,
        &mut current_tour,
        &mut node_queue,
        &mut rng,
    );
}

fn lin_kernighan(
    distances: &SquareMatrix<Distance>,
    edge_state: &mut SquareMatrix<EdgeState>,
    current_tour: &mut CurrentTour,
    node_queue: &mut VecDeque<Node>,
    rng: &mut impl Rng,
) {
    let mut delta = 0.0;
    let mut total_win = 0.0;

    while let Some(node) = node_queue.pop_front() {
        delta = lin_kernighan_step_seq(distances, edge_state, current_tour, node_queue, rng, node);
    }
}

/// Performs a single step of the Lin-Kernighan heuristic, starting from the given current node.
fn lin_kernighan_step_seq(
    distances: &SquareMatrix<Distance>,
    edge_state: &mut SquareMatrix<EdgeState>,
    current_tour: &mut CurrentTour,
    node_queue: &mut VecDeque<Node>,
    rng: &mut impl Rng,
    current_node: Node,
) -> f64 {
    let next_node = current_tour.successor(current_node);

    edge_state.set_data_symmetric(current_node, next_node, EdgeState::Deleted);
    let gain = distances.get_data(current_node, next_node);

    lin_kernighan_step(
        distances,
        edge_state,
        current_tour,
        node_queue,
        rng,
        current_node,
        next_node,
        gain,
    );

    edge_state.set_data_symmetric(current_node, next_node, EdgeState::None);

    0.0
}

fn lin_kernighan_step(
    distances: &SquareMatrix<Distance>,
    edge_state: &mut SquareMatrix<EdgeState>,
    current_tour: &mut CurrentTour,
    node_queue: &mut VecDeque<Node>,
    rng: &mut impl Rng,
    current_node: Node,
    next_node: Node,
    gain: Distance,
) {
    // let candidate_edges = 
}

/// 
fn find_candidate_edges(
    distances: &SquareMatrix<Distance>,
    edge_state: &SquareMatrix<EdgeState>,
    current_node: Node,
) -> Vec<Node> {
    let mut candidates = Vec::new();
    let adjacency_list = distances.get_adjacency_list(current_node);
    for (to_index, &distance) in adjacency_list.iter().enumerate() {
        let to_node = Node(to_index);
        if edge_state.get_data(current_node, to_node) == EdgeState::None {
            candidates.push(to_node);
        }
    }
    candidates
}

fn find_random_tour(dimension: usize, rng: &mut impl Rng) -> Vec<Node> {
    let mut vec = (0..dimension).map(Node).collect::<Vec<_>>();
    vec.shuffle(rng);
    vec
}

fn initialize_current_tour(dimension: usize) -> CurrentTour {
    CurrentTour {
        nodes: (0..dimension).map(Node).collect(),
    }
}
