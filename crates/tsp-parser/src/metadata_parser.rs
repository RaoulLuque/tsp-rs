use std::{
    fs::File,
    io::{BufReader, Lines},
};

use tsp_core::{
    instance::InstanceMetadata,
    tsp_lib_spec::{
        DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType,
        ProblemType, TSPDataKeyword, TSPSpecificationKeyword,
    },
};

use thiserror::Error;

use crate::ParserError;

#[derive(Error, Debug)]
pub enum MetaDataParseError {
    #[error("Invalid keyword in this line: {0}")]
    InvalidKeyword(String),
    #[error("Invalid (problem) TYPE value: {0}")]
    InvalidProblemType(String),
    #[error("Invalid DIMENSION value: {0}")]
    InvalidDimension(String),
    #[error("Invalid EDGE_WEIGHT_TYPE value: {0}")]
    InvalidEdgeWeightType(String),
    #[error("Invalid EDGE_WEIGHT_FORMAT value: {0}")]
    InvalidEdgeWeightFormat(String),
    #[error("Invalid EDGE_DATA_FORMAT value: {0}")]
    InvalidEdgeDataFormat(String),
    #[error("Invalid NODE_COORD_TYPE value: {0}")]
    InvalidNodeCoordType(String),
    #[error("Invalid DISPLAY_DATA_TYPE value: {0}")]
    InvalidDisplayDataType(String),
}

enum TSPSpecificationOrDataKeyword {
    SpecificationKeyword(TSPSpecificationKeyword),
    DataKeyword(TSPDataKeyword),
}

pub fn parse_metadata(
    input: &mut Lines<BufReader<File>>,
) -> Result<(InstanceMetadata, &Lines<BufReader<File>>), ParserError> {
    let metadata_builder = InstanceMetadata::builder();
    let a = while let Some(Ok(line)) = input.next() {
        todo!()
    };
    todo!()
}

fn parse_specification_or_data_keyword(
    line: &str,
) -> Result<TSPSpecificationOrDataKeyword, ParserError> {
    let mut parts = line.splitn(2, ':');
    match (parts.next(), parts.next()) {
        // Hot path
        (Some(k), Some(v)) => Ok(TSPSpecificationOrDataKeyword::SpecificationKeyword(
            parse_specification(k.trim_end(), v.trim_end())?,
        )),
        // Cold path(s)
        (Some(k), None) => {
            return Ok(TSPSpecificationOrDataKeyword::DataKeyword(
                parse_data_keyword(k)?,
            ));
        }
        _ => return Err(MetaDataParseError::InvalidKeyword(line.to_string()).into()),
    }
}

fn parse_specification(keyword: &str, value: &str) -> Result<TSPSpecificationKeyword, ParserError> {
    match keyword {
        "NAME" => Ok(TSPSpecificationKeyword::NAME(value.to_string())),
        "TYPE" => Ok(TSPSpecificationKeyword::TYPE(parse_problem_type(value)?)),
        "COMMENT" => Ok(TSPSpecificationKeyword::COMMENT(value.to_string())),
        "DIMENSION" => {
            Ok(TSPSpecificationKeyword::DIMENSION(value.parse().map_err(
                |_| MetaDataParseError::InvalidDimension(keyword.to_string()),
            )?))
        }
        "CAPACITY" => Ok(TSPSpecificationKeyword::CAPACITY(value.parse().map_err(
            |_| MetaDataParseError::InvalidDimension(keyword.to_string()),
        )?)),
        "EDGE_WEIGHT_TYPE" => Ok(TSPSpecificationKeyword::EDGE_WEIGHT_TYPE(
            parse_edge_weight_type(value)?,
        )),
        "EDGE_WEIGHT_FORMAT" => Ok(TSPSpecificationKeyword::EDGE_WEIGHT_FORMAT(
            parse_edge_weight_format(value)?,
        )),
        "EDGE_DATA_FORMAT" => Ok(TSPSpecificationKeyword::EDGE_DATA_FORMAT(
            parse_edge_data_format(value)?,
        )),
        "NODE_COORD_TYPE" => Ok(TSPSpecificationKeyword::NODE_COORD_TYPE(
            parse_node_coord_type(value)?,
        )),
        "DISPLAY_DATA_TYPE" => Ok(TSPSpecificationKeyword::DISPLAY_DATA_TYPE(
            parse_display_data_type(value)?,
        )),
        _ => Err(MetaDataParseError::InvalidKeyword(keyword.to_string()).into()),
    }
}

fn parse_data_keyword(input: &str) -> Result<TSPDataKeyword, ParserError> {
    match input {
        "NODE_COORD_SECTION" => Ok(TSPDataKeyword::NODE_COORD_SECTION),
        "DEPOT_SECTION" => Ok(TSPDataKeyword::DEPOT_SECTION),
        "DEMAND_SECTION" => Ok(TSPDataKeyword::DEMAND_SECTION),
        "EDGE_DATA_SECTION" => Ok(TSPDataKeyword::EDGE_DATA_SECTION),
        "FIXED_EDGES_SECTION" => Ok(TSPDataKeyword::FIXED_EDGES_SECTION),
        "DISPLAY_DATA_SECTION" => Ok(TSPDataKeyword::DISPLAY_DATA_SECTION),
        "TOUR_SECTION" => Ok(TSPDataKeyword::TOUR_SECTION),
        "EDGE_WEIGHT_SECTION" => Ok(TSPDataKeyword::EDGE_WEIGHT_SECTION),
        _ => Err(MetaDataParseError::InvalidKeyword(input.to_string()).into()),
    }
}

fn parse_problem_type(input: &str) -> Result<ProblemType, ParserError> {
    match input {
        "TSP" => Ok(ProblemType::TSP),
        "ATSP" => Ok(ProblemType::ATSP),
        "SOP" => Ok(ProblemType::SOP),
        "HCP" => Ok(ProblemType::HCP),
        "TOUR" => Ok(ProblemType::TOUR),
        _ => Err(MetaDataParseError::InvalidProblemType(input.to_string()).into()),
    }
}

fn parse_edge_weight_type(input: &str) -> Result<EdgeWeightType, ParserError> {
    match input {
        "EXPLICIT" => Ok(EdgeWeightType::EXPLICIT),
        "EUC_2D" => Ok(EdgeWeightType::EUC_2D),
        "EUC_3D" => Ok(EdgeWeightType::EUC_3D),
        "MAX_2D" => Ok(EdgeWeightType::MAX_2D),
        "MAX_3D" => Ok(EdgeWeightType::MAX_3D),
        "MAN_2D" => Ok(EdgeWeightType::MAN_2D),
        "MAN_3D" => Ok(EdgeWeightType::MAN_3D),
        "CEIL_2D" => Ok(EdgeWeightType::CEIL_2D),
        "GEO" => Ok(EdgeWeightType::GEO),
        "ATT" => Ok(EdgeWeightType::ATT),
        "XRAY1" => Ok(EdgeWeightType::XRAY1),
        "XRAY2" => Ok(EdgeWeightType::XRAY2),
        "SPECIAL" => Ok(EdgeWeightType::SPECIAL),
        _ => Err(MetaDataParseError::InvalidEdgeWeightType(input.to_string()).into()),
    }
}

fn parse_edge_weight_format(input: &str) -> Result<EdgeWeightFormat, ParserError> {
    match input {
        "FUNCTION" => Ok(EdgeWeightFormat::FUNCTION),
        "FULL_MATRIX" => Ok(EdgeWeightFormat::FULL_MATRIX),
        "UPPER_ROW" => Ok(EdgeWeightFormat::UPPER_ROW),
        "LOWER_ROW" => Ok(EdgeWeightFormat::LOWER_ROW),
        "UPPER_DIAG_ROW" => Ok(EdgeWeightFormat::UPPER_DIAG_ROW),
        "LOWER_DIAG_ROW" => Ok(EdgeWeightFormat::LOWER_DIAG_ROW),
        "UPPER_COL" => Ok(EdgeWeightFormat::UPPER_COL),
        "LOWER_COL" => Ok(EdgeWeightFormat::LOWER_COL),
        "UPPER_DIAG_COL" => Ok(EdgeWeightFormat::UPPER_DIAG_COL),
        "LOWER_DIAG_COL" => Ok(EdgeWeightFormat::LOWER_DIAG_COL),
        _ => Err(MetaDataParseError::InvalidEdgeWeightFormat(input.to_string()).into()),
    }
}

fn parse_edge_data_format(input: &str) -> Result<EdgeDataFormat, ParserError> {
    match input {
        "EDGE_LIST" => Ok(EdgeDataFormat::EDGE_LIST),
        "ADJ_LIST" => Ok(EdgeDataFormat::ADJ_LIST),
        _ => Err(MetaDataParseError::InvalidEdgeDataFormat(input.to_string()).into()),
    }
}

fn parse_node_coord_type(input: &str) -> Result<NodeCoordType, ParserError> {
    match input {
        "TWOD_COORDS" => Ok(NodeCoordType::TWOD_COORDS),
        "THREED_COORDS" => Ok(NodeCoordType::THREED_COORDS),
        "NO_COORDS" => Ok(NodeCoordType::NO_COORDS),
        _ => Err(MetaDataParseError::InvalidNodeCoordType(input.to_string()).into()),
    }
}

fn parse_display_data_type(input: &str) -> Result<DisplayDataType, ParserError> {
    match input {
        "COORD_DISPLAY" => Ok(DisplayDataType::COORD_DISPLAY),
        "TWOD_DISPLAY" => Ok(DisplayDataType::TWOD_DISPLAY),
        "NO_DISPLAY" => Ok(DisplayDataType::NO_DISPLAY),
        _ => Err(MetaDataParseError::InvalidDisplayDataType(input.to_string()).into()),
    }
}
