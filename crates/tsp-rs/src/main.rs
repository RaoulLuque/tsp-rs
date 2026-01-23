use std::ops::Add;

use tsp_core::instance::{
    distance::Distance,
    matrix::{SquareMatrix, TriangularMatrix},
};
use tsp_solvers::{held_karp, held_karp_mod::held_karp_parallel};

fn main() {
    env_logger::init();

    let tsp_instance = tsp_parser::parse_tsp_instance::<TriangularMatrix<Distance>>(
        "instances/tsplib_symmetric/bays29.tsp",
    )
    .unwrap();
    let dists_as_integer = tsp_instance
        .distance_matrix()
        .data()
        .iter()
        .map(|d| d.0 as u32)
        .collect();

    let tsp_instance_as_integer =
        TriangularMatrix::<u32>::new(dists_as_integer, tsp_instance.metadata().dimension);
    println!(
        "TSP Instance as integer Parsed: {}",
        tsp_instance_as_integer
    );
    // let best_tour = held_karp(tsp_instance.distance_matrix());
    // if let Some(best_tour) = &best_tour {
    //     println!("Best tour found: {:?}", best_tour.cost.0);
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
