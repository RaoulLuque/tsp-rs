use tsp_core::instance::{UnTour, distance::Distance, edge::UnEdge, node::Node};
use tsp_solvers::held_karp;

#[test]
fn test_held_karp_on_12() {
    let tsp_instance = tsp_parser::parse_tsp_instance("../../instances/tsp_rust/12.tsp").unwrap();
    let distances_non_symmetric = tsp_instance.distance_matrix().to_edge_data_matrix();
    let best_tour = held_karp(&distances_non_symmetric).unwrap();
    let edges = vec![
        UnEdge {
            from: Node(1),
            to: Node(6),
        },
        UnEdge {
            from: Node(6),
            to: Node(2),
        },
        UnEdge {
            from: Node(1),
            to: Node(10),
        },
        UnEdge {
            from: Node(10),
            to: Node(3),
        },
        UnEdge {
            from: Node(2),
            to: Node(4),
        },
        UnEdge {
            from: Node(4),
            to: Node(9),
        },
        UnEdge {
            from: Node(9),
            to: Node(11),
        },
        UnEdge {
            from: Node(11),
            to: Node(5),
        },
        UnEdge {
            from: Node(3),
            to: Node(7),
        },
        UnEdge {
            from: Node(7),
            to: Node(8),
        },
        UnEdge {
            from: Node(0),
            to: Node(5),
        },
        UnEdge {
            from: Node(0),
            to: Node(8),
        },
    ];
    let expected_tour = UnTour {
        edges,
        cost: Distance(1200),
    };
    assert_eq!(best_tour, expected_tour);
}
