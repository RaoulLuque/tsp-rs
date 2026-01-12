use std::ops::Add;

use tsp_core::instance::{
    distance::Distance,
    matrix::{Matrix, MatrixSym},
};
use tsp_solvers::{held_karp, held_karp_mod::held_karp_parallel};

fn main() {
    env_logger::init();

    let tsp_instance =
        tsp_parser::parse_tsp_instance::<Matrix<Distance>>("instances/tsplib_symmetric/st70.tsp")
            .unwrap();
    let best_tour = held_karp_parallel(tsp_instance.distance_matrix());
    if let Some(best_tour) = &best_tour {
        println!("Best tour found: {:?}", best_tour.cost.0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
