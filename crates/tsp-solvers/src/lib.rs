/*!
This crate provides implementations of various algorithms to solve the Traveling Salesperson Problem (TSP).
Explanations and references for the algorithms can be found in their respective modules.
 */
#![warn(missing_debug_implementations, missing_docs)]

pub mod held_karp_mod;
pub mod lin_kernighan_mod;
pub use held_karp_mod::held_karp;
