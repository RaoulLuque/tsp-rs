pub(crate) mod square;
pub(crate) mod triangular;
pub use square::{MatrixViewZeroRemoved, SquareMatrix};
pub use triangular::{
    TriangularMatrix, get_lower_triangle_matrix_entry, get_lower_triangle_matrix_entry_row_bigger,
};
