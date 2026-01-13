use std::ops::Add;

use tsp_core::instance::{
    distance::Distance,
    matrix::{Matrix, MatrixSym},
};
use tsp_solvers::{held_karp, held_karp_mod::held_karp_parallel};

fn main() {
    env_logger::init();

    let tsp_instance: tsp_core::instance::TSPSymInstance<MatrixSym<Distance>> =
        tsp_parser::parse_tsp_instance::<MatrixSym<Distance>>(
            "instances/tsplib_symmetric/ulysses22.tsp",
        )
        .unwrap();
    println!(
        "{:?}",
        tsp_instance
            .raw_distances()
            .iter()
            .map(|dist| dist.0)
            .collect::<Vec<i32>>()
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
