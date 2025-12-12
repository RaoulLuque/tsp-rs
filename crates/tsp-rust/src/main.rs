fn main() {
    let tsp_instance = tsp_parser::parse_tsp_instance("instances/bench/d198.tsp").unwrap();
    // println!("Parsed TSP instance: {:?}", tsp_instance.raw_distances());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
