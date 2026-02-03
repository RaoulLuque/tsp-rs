#![warn(missing_debug_implementations, missing_docs)]

pub mod instance;
pub mod tsp_lib_spec;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works_short() {
        assert_eq!(2 + 2, 4);
    }
}
