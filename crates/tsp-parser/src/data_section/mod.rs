/// Module for parsing distance data sections from TSP instance files.
///
/// According to TSPLIB95, distance data can be represented in various formats,
/// such as explicit distance matrices or coordinate-based representations.
///
/// Distance values are required to be non-negative integers. Computations are expected to be
/// carried out in double precision arithmetic, i.e. `f64` in Rust.
use memchr::memchr;
use memmap2::Mmap;
use tsp_core::{
    instance::InstanceMetadata,
    tsp_lib_spec::{EdgeWeightType, TSPDataKeyword},
};

pub(crate) mod distance_function;

use crate::{
    data_section::distance_function::{euclidean_distance_2d, geographical_distance},
    distance_container::ParseFromTSPLib,
};

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// A point in 3D space.
#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A point in geographical latitude and longitude radiance coordinates.
#[derive(Debug, Clone, Copy)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn parse_data_sections<DistanceContainer: ParseFromTSPLib>(
    mmap: &Mmap,
    index_in_map: &mut usize,
    data_keyword: TSPDataKeyword,
    metadata: &InstanceMetadata,
) -> DistanceContainer {
    match metadata.edge_weight_type {
        // The distance function is not chosen via a match statement here because the compiler
        // does not seem to inline the distance function properly in that case
        // (leading to a big performance hit)
        EdgeWeightType::EUC_2D => {
            let distance_function = euclidean_distance_2d;
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::MAX_2D => {
            let distance_function = distance_function::max_distance_2d;
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::MAN_2D => {
            let distance_function = distance_function::manhattan_distance_2d;
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::CEIL_2D => {
            let distance_function = distance_function::ceil_distance_2d;
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::ATT => {
            let distance_function = distance_function::att_distance_2d;
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::GEO => {
            let distance_function = geographical_distance;
            // TODO(perf): Possibly parallelize the conversion to geo coordinates
            let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata)
                .into_iter()
                .map(|point| distance_function::convert_to_geo_coordinates(&point))
                .collect::<Vec<GeoPoint>>();
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::EUC_3D => {
            let distance_function = distance_function::euclidean_distance_3d;
            let node_data = parse_3d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::MAX_3D => {
            let distance_function = distance_function::max_distance_3d;
            let node_data = parse_3d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::MAN_3D => {
            let distance_function = distance_function::manhattan_distance_3d;
            let node_data = parse_3d_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        EdgeWeightType::EXPLICIT => {
            // TODO: Implement explicit distance matrix parsing
            todo!("Explicit distance matrix parsing is not yet implemented");
        }
        _ => unimplemented!(
            "Node coordinate type {:?} is not yet implemented",
            metadata.node_coord_type
        ),
    }
}

fn parse_2d_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> Vec<Point2D> {
    let mut point_data: Vec<Point2D> = Vec::with_capacity(metadata.dimension);

    // Read a line to test if the point data is floating point or integer
    let is_float_data = is_float_data(mmap, index_in_map);

    while let Some(index_newline) = memchr(b'\n', &mmap[*index_in_map..]) {
        let line = &mmap[*index_in_map..*index_in_map + index_newline];
        // SAFETY: The TSP instance file is expected to be valid UTF-8
        let line_str = unsafe { std::str::from_utf8_unchecked(line) };

        // Move the index to the start of the next line (+1 for the newline character)
        *index_in_map += index_newline + 1;

        // Check if end of file is reached
        if line_str == "EOF" {
            break;
        }

        let point = parse_line_to_2d_point(line_str, is_float_data);

        point_data.push(point);
    }

    point_data
}

fn parse_3d_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> Vec<Point3D> {
    let mut point_data: Vec<Point3D> = Vec::with_capacity(metadata.dimension);

    // Read a line to test if the point data is floating point or integer
    let is_float_data = is_float_data(mmap, index_in_map);

    while let Some(index_newline) = memchr(b'\n', &mmap[*index_in_map..]) {
        let line = &mmap[*index_in_map..*index_in_map + index_newline];
        // SAFETY: The TSP instance file is expected to be valid UTF-8
        let line_str = unsafe { std::str::from_utf8_unchecked(line) };

        // Move the index to the start of the next line (+1 for the newline character)
        *index_in_map += index_newline + 1;

        // Check if end of file is reached
        if line_str == "EOF" {
            break;
        }

        let point = parse_line_to_3d_point(line_str, is_float_data);

        point_data.push(point);
    }

    point_data
}

#[inline(always)]
fn parse_line_to_2d_point(line_str: &str, is_float_data: bool) -> Point2D {
    // We assume the input to be split by ascii whitespace
    let mut parts = line_str.split_ascii_whitespace();
    let _node_index = parts.next();

    let x_str = parts.next().expect("Missing x coordinate");
    let y_str = parts.next().expect("Missing y coordinate");
    let (x, y) = if is_float_data {
        (
            x_str
                .parse::<f64>()
                .expect("x coordinate should always be a valid f64 floating point number"),
            y_str
                .parse::<f64>()
                .expect("y coordinate should always be a valid f64 floating point number"),
        )
    } else {
        (
            x_str
                .parse::<u64>()
                .expect("x coordinate should be a valid u64 integer by sampling first line")
                as f64,
            y_str
                .parse::<u64>()
                .expect("y coordinate should be a valid u64 integer by sampling first line")
                as f64,
        )
    };

    Point2D { x, y }
}

#[inline(always)]
fn parse_line_to_3d_point(line_str: &str, is_float_data: bool) -> Point3D {
    // We assume the input to be split by ascii whitespace
    let mut parts = line_str.split_ascii_whitespace();
    let _node_index = parts.next();

    let x_str = parts.next().expect("Missing x coordinate");
    let y_str = parts.next().expect("Missing y coordinate");
    let z_str = parts.next().expect("Missing z coordinate");
    let (x, y, z) = if is_float_data {
        (
            x_str
                .parse::<f64>()
                .expect("x coordinate should always be a valid f64 floating point number"),
            y_str
                .parse::<f64>()
                .expect("y coordinate should always be a valid f64 floating point number"),
            z_str
                .parse::<f64>()
                .expect("z coordinate should always be a valid f64 floating point number"),
        )
    } else {
        (
            x_str
                .parse::<u64>()
                .expect("x coordinate should be a valid u64 integer by sampling first line")
                as f64,
            y_str
                .parse::<u64>()
                .expect("y coordinate should be a valid u64 integer by sampling first line")
                as f64,
            z_str
                .parse::<u64>()
                .expect("z coordinate should be a valid u64 integer by sampling first line")
                as f64,
        )
    };

    Point3D { x, y, z }
}

#[inline(always)]
fn is_float_data(mmap: &Mmap, index_in_map: &usize) -> bool {
    let index_newline =
        memchr(b'\n', &mmap[*index_in_map..]).expect("The data section should not be empty");
    let line = &mmap[*index_in_map..*index_in_map + index_newline];

    // SAFETY: The TSP instance file is expected to be valid UTF-8
    let line_str = unsafe { std::str::from_utf8_unchecked(line) };

    // We assume the input to be split by ascii whitespace
    let mut parts = line_str.split_ascii_whitespace();
    let _node_index = parts.next();

    let x_str = parts.next().expect("Missing x coordinate");
    x_str.contains('.')
}
