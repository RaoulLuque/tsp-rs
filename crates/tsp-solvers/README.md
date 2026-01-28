# TSP Solvers

This crate provides implementations of various algorithms to solve the Traveling Salesman Problem (TSP).
The different algorithms are implemented as separate modules within this crate.

[![Build status](https://github.com/RaoulLuque/tsp-rs/workflows/ci/badge.svg)](https://github.com/RaoulLuque/tsp-rs/actions)
[![](https://img.shields.io/crates/v/tsp-solvers.svg)](https://crates.io/crates/tsp-solvers)

Dual-licensed under [MIT](../../LICENSE-MIT) or the [Apache 2.0 License](../../LICENSE-APACHE).

### Documentation

[https://docs.rs/tsp-solvers](https://docs.rs/tsp-solvers)


### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tsp-solvers = "0.1"
```

## License

This crate is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option. This is done to be compatible with the Rust ecosystem.

Some of the solvers and algorithms implemented in this crate are based on or inspired by existing works.
Therefore, the following solvers are licensed under only the MIT License. These include the
[Held-Karp implementation](./crates/tsp-solvers/src/held_karp_mod/) and
the [Lin-Kernighan implementation](./crates/tsp-solvers/src/lin_kernighan_mod/).
