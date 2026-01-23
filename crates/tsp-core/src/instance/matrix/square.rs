use std::fmt::Display;

use crate::instance::node::Node;

#[derive(Debug, Clone)]
/// Row-major full (square) matrix to store arbitrary edge data.
///
/// Can store both symmetric and asymmetric data.
///
/// The underlying data storage is guaranteed to have length dimension * dimension.
/// That is, data from node i (row) to node j (column) is at index (i * dimension + j).
pub struct SquareMatrix<Data> {
    data: Vec<Data>,
    dimension: usize,
}

impl<Data> SquareMatrix<Data> {
    /// Create a new EdgeDataMatrix from raw data and dimension.
    ///
    /// Panics if the length of data does not equal dimension * dimension.
    pub fn new(data: Vec<Data>, dimension: usize) -> SquareMatrix<Data> {
        assert_eq!(data.len(), dimension * dimension);
        SquareMatrix { data, dimension }
    }

    /// Returns the dimension of the matrix. That is, the number of nodes, which is the same as the
    /// number of rows and columns.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns a reference to the underlying data.
    ///
    /// It is guaranteed that the length of the data is dimension * dimension.
    pub fn data(&self) -> &[Data] {
        &self.data
    }

    /// Create a new EdgeDataMatrix from a distance function.
    ///
    /// The distance function must not necessarily be symmetric.
    pub fn new_from_distance_function(
        dimension: usize,
        distance_function: impl Fn(Node, Node) -> Data,
    ) -> Self {
        let data: Vec<_> = (0..dimension)
            .flat_map(|row| (0..dimension).map(move |column| (Node(row), Node(column))))
            .map(|(from, to)| distance_function(from, to))
            .collect();

        SquareMatrix::new(data, dimension)
    }
}

impl<Data: Clone> SquareMatrix<Data> {
    /// Create a new EdgeDataMatrix from dimension, filling all entries with the given value.
    pub fn new_from_dimension_with_value(dimension: usize, value: Data) -> Self {
        let size = dimension * dimension;
        SquareMatrix::new(vec![value; size], dimension)
    }
}

impl<Data: Copy> SquareMatrix<Data> {
    /// Access the data at (from, to).
    #[inline(always)]
    pub fn get_data(&self, from: Node, to: Node) -> Data {
        let index = self.get_index(from, to);
        self.data[index]
    }

    /// Access the data in a way that allows for faster sequential access when iterating over 'to'
    /// nodes.
    ///
    /// For a similar function that allows for faster sequential access when iterating over 'from'
    /// nodes, please switch the arguments to this function. This only works, if the respective
    /// matrix entries are symmetric.
    #[inline(always)]
    pub fn get_data_to_seq(&self, from: Node, to: Node) -> Data {
        let index = self.get_index(from, to);
        self.data[index]
    }

    /// Get the adjacency list for a given 'from' node.
    #[inline(always)]
    pub fn get_adjacency_list(&self, from: Node) -> &[Data] {
        let start_index = self.get_index(from, Node(0));
        &self.data[start_index..start_index + self.dimension]
    }

    /// Set data symmetrically. That is, sets both (from, to) and (to, from).
    #[inline(always)]
    pub fn set_data_symmetric(&mut self, from: Node, to: Node, data: Data) {
        self.set_data(from, to, data);
        self.set_data(to, from, data);
    }
}

impl<Data> SquareMatrix<Data> {
    /// Set data asymmetrically. That is, set only the entry in row 'from' and column 'to'.
    #[inline(always)]
    pub fn set_data(&mut self, from: Node, to: Node, data: Data) {
        let index = self.get_index(from, to);
        self.data[index] = data;
    }

    /// Split the matrix into a zero row and a zero-removed matrix.
    ///
    /// The returned zero row is of length dimension.
    pub fn split_first_row<'a>(&'a self) -> (&'a [Data], MatrixViewZeroRemoved<'a, Data>) {
        let zero_row = &self.data[0..self.dimension];
        let zero_removed = MatrixViewZeroRemoved {
            data: &self.data[self.dimension..],
            dimension: self.dimension,
        };
        (zero_row, zero_removed)
    }

    /// Get the index in the underlying data vector for the given (from, to) pair.
    #[inline(always)]
    fn get_index(&self, from: Node, to: Node) -> usize {
        from.0 * self.dimension + to.0
    }
}

impl<Data: Display + Ord + Copy> Display for SquareMatrix<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_value = self
            .data
            .iter()
            .max()
            .expect("Matrix should have at least one entry for display");
        let max_len = format!("{}", max_value).len();
        println!("Max len   : {}", max_len);
        for row in 0..self.dimension {
            for column in 0..self.dimension {
                let value = self.get_data(Node(row), Node(column));
                write!(f, "{:max_len$} ", value)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// View of an [EdgeDataMatrix] with the zero-eth row removed.
///
/// I.e. a (n-1) x n matrix where row 0 corresponds to node 1, row 1 to node 2, ..., row n-1 to node
/// n. Data is borrowed from the original matrix, i.e. its lifetime is tied to that of the original
/// matrix and immutable.
#[derive(Debug)]
pub struct MatrixViewZeroRemoved<'a, Data> {
    data: &'a [Data],
    dimension: usize,
}

impl<'a, Data: Copy> MatrixViewZeroRemoved<'a, Data> {
    /// Get the adjusted dimension (i.e., n-1 if the dimension of the underlying matrix is n).
    pub fn dimension_adjusted(&self) -> usize {
        self.dimension - 1
    }

    /// Get the total dimension (i.e., n if the dimension of the underlying matrix is n).
    pub fn dimension_total(&self) -> usize {
        self.dimension
    }

    /// Get the adjacency list for a given 'from' node. Assumes `from` is not node 0, i.e., starts
    /// at 1. That is, takes into account that the zero row/column has been removed.
    #[inline(always)]
    pub fn get_adjacency_list(&self, from: Node) -> &[Data] {
        debug_assert!(from.0 >= 1);
        let start_index = (from.0 - 1) * self.dimension;
        &self.data[start_index..start_index + self.dimension]
    }
}
