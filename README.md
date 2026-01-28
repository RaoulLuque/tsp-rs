# Traveling Salescrab Problem

<p align="center">
  <img src="assets/images/traveling_salescrab.png" alt="Traveling Salescrab"/>
</p>

This repository contains different crates for solving the [Traveling Salesperson Problem (TSP)](https://en.wikipedia.org/wiki/Travelling_salesman_problem) in the [Rust Programming Language](https://rust-lang.org/). The TSP is a classic optimization problem where the goal is to find the shortest possible route that visits a set of cities and returns to the origin city.

## References and Credit

- [The Traveling Salesman Problem: A Computational Study](https://www.degruyterbrill.com/document/doi/10.1515/9781400841103/html?lang=en)
  by David L. Applegate, Robert E. Bixby, Vasek Chvatal, and William J. Cook.
  This book provides an in-depth treatment of various TSP algorithms.
- [Concorde TSP Solver](https://www.math.uwaterloo.ca/tsp/concorde.html): The Concorde TSP solver
  is a well-known implementation of TSP algorithms.

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option. This is done to be compatible with the Rust ecosystem.

Some of the solvers and algorithms implemented in this repository are based on or inspired by existing works.
Therefore, the following solvers are licensed under only the MIT License. These include the
[Held-Karp implementation](./crates/tsp-solvers/src/held_karp_mod/) and
the [Lin-Kernighan implementation](./crates/tsp-solvers/src/lin_kernighan_mod/).

Furthermore, the instances in the `instances/tsplib_symmetric/` directory are redistributions of the original TSPLIB data and are provided here for convenience. They are not authored by the developers of this project, and the TSPLIB instances themselves are not
covered by this project’s MIT/Apache-2.0 license. All credit for these instances goes to the original author.

### Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
