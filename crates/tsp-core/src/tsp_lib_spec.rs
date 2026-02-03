#![allow(non_camel_case_types)]

/// Specifies the different data sections that can appear in a TSP problem instance file.
#[derive(Debug, Clone)]
pub enum TSPDataKeyword {
    /// Contains a list of node coordinates.
    ///
    /// Each line is of the form: `<node id: integer> <x coord: float> <y coord: float>` (or
    /// similarly for 3D coordinates).
    NODE_COORD_SECTION,
    /// Contains a list of possible alternate depot nodes. The list is terminated by -1.
    DEPOT_SECTION,
    /// Contains a list of demands for each node.
    ///
    /// Each line is of the form: `<node id: integer> <demand: integer>`.
    ///
    /// Depot nodes must also occur in this section with demand 0.
    DEMAND_SECTION,
    /// Contains a list of explicit edge data.
    ///
    /// Is in either of the two [EdgeDataFormat]s.
    ///
    /// If [EdgeDataFormat::EDGE_LIST] is used, each line is of the form: `<node1 id: integer>
    /// <node2 id: integer>`.
    ///
    /// If [EdgeDataFormat::ADJ_LIST] is used, each line is of the form: `<node id: integer>
    /// <adjacent node1 id: integer> <adjacent node2 id: integer> ... -1`. That is, the list of
    /// adjacent nodes is terminated by -1.
    EDGE_DATA_SECTION,
    /// Contains a list of fixed edges that must be included in the tour.
    ///
    /// Each line is of the form: `<node1 id: integer> <node2 id: integer>`.
    /// This section is terminated by -1.
    FIXED_EDGES_SECTION,
    /// If [DisplayDataType::TWOD_DISPLAY] is used, contains a list of 2D coordinates for display.
    ///
    /// Each line is of the form: `<node id: integer> <x coord: float> <y coord: float>`.
    DISPLAY_DATA_SECTION,
    /// Contains a list of tours.
    ///
    /// Each tour is given by a list of node ids, terminated by -1.
    TOUR_SECTION,
    /// Contains the edge weight matrix or list, if [EdgeWeightType::EXPLICIT] is used.
    ///
    /// The format is specified by [EdgeWeightFormat].
    EDGE_WEIGHT_SECTION,
}

/// Specifies the type of problem instance.
#[derive(Debug, Clone)]
pub enum ProblemType {
    /// Symmetric Traveling Salesperson Problem.
    TSP,
    /// Asymmetric Traveling Salesperson Problem.
    ATSP,
    /// Sequential Ordering Problem.
    SOP,
    /// Hamiltonian Cycle Problem.
    HCP,
    /// Capacitated Vehicle Routing Problem.
    CVRP,
    /// Collection of Traveling Salesperson Problems.
    TOUR,
}

/// Specifies how edge weights are provided in the problem instance.
#[derive(Debug, Clone)]
pub enum EdgeWeightType {
    /// Weights are provided explicitly, see [EdgeWeightFormat].
    EXPLICIT,
    /// Weights are the Euclidean distance in 2D.
    EUC_2D,
    /// Weights are the Euclidean distance in 3D.
    EUC_3D,
    /// Weights are the maximum distance in 2D.
    MAX_2D,
    /// Weights are the maximum distance in 3D.
    MAX_3D,
    /// Weights are the Manhattan distance in 2D.
    MAN_2D,
    /// Weights are the Manhattan distance in 3D.
    MAN_3D,
    /// Weights are the ceiling of the Euclidean distance in 2D.
    CEIL_2D,
    /// Weights are the geographical distances as spcified in TSPLIB.
    GEO,
    /// Weights have a special distance function as specified in TSPLIB (only applicable for att
    /// instances).
    ATT,
    /// Weights have a special distance function for crystallography problems as specified in
    /// TSPLIB.
    XRAY1,
    /// Weights have a special distance function for crystallography problems as specified in
    /// TSPLIB.
    XRAY2,
    /// Weights have a special distance function documented "elsewhere".
    SPECIAL,
}

/// Specifies the format in which edge weights are provided, if they are explicit (i.e. via
/// [EdgeWeightType::EXPLICIT]).
#[derive(Debug, Clone)]
pub enum EdgeWeightFormat {
    /// Weights are given by a function, i.e. [EdgeWeightType] is not [EdgeWeightType::EXPLICIT].
    FUNCTION,
    /// Weights are given by a full matrix.
    FULL_MATRIX,
    /// Weights are given by an upper triangular matrix (row-wise without diagonal entires).
    UPPER_ROW,
    /// Weights are given by a lower triangular matrix (row-wise without diagonal entires).
    LOWER_ROW,
    /// Weights are given by an upper triangular matrix (row-wise with diagonal entires).
    UPPER_DIAG_ROW,
    /// Weights are given by a lower triangular matrix (row-wise with diagonal entires).
    LOWER_DIAG_ROW,
    /// Weights are given by an upper triangular matrix (column-wise without diagonal entires).
    UPPER_COL,
    /// Weights are given by a lower triangular matrix (column-wise without diagonal entires).
    LOWER_COL,
    /// Weights are given by an upper triangular matrix (column-wise with diagonal entires).
    UPPER_DIAG_COL,
    /// Weights are given by a lower triangular matrix (column-wise with diagonal entires).
    LOWER_DIAG_COL,
}

/// Specifies the format in which edge data is provided, if the graph is not complete.
#[derive(Debug, Clone)]
pub enum EdgeDataFormat {
    /// The edge data is provided as an edge list.
    EDGE_LIST,
    /// The edge data is provided as an adjacency list.
    ADJ_LIST,
}

/// Specifies the type of coordinates provided for the nodes.
#[derive(Debug, Clone, Default)]
pub enum NodeCoordType {
    /// 2D coordinates are provided.
    TWOD_COORDS,
    /// 3D coordinates are provided.
    THREED_COORDS,
    /// The nodes do not have associated coordinates.
    #[default]
    NO_COORDS,
}

/// Specifies how a graphical display of the problem can be generated.
///
/// If node coordinates are provided, the default value is COORD_DISPLAY, otherwise NO_DISPLAY.
#[derive(Debug, Clone)]
pub enum DisplayDataType {
    /// The node coordinates can be used for display.
    COORD_DISPLAY,
    /// 2D coordinates for display are provided.
    TWOD_DISPLAY,
    /// No display data is provided.
    NO_DISPLAY,
}
