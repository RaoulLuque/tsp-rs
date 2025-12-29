use tsp_solvers::held_karp;

fn main() {
    let tsp_instance = tsp_parser::parse_tsp_instance("instances/bench/berlin52.tsp").unwrap();
    // println!("Parsed TSP instance: {:?}", tsp_instance.raw_distances());
    let distances_non_symmetric = tsp_instance.distances().to_non_symmetric();
    let best_tour = held_karp(&distances_non_symmetric);
    println!("Best tour found: {:?}", best_tour);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
