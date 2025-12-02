use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use thiserror::Error;
use tsp_core::instance::TSPInstance;

use crate::metadata_parser::{MetaDataParseError, parse_metadata};

pub mod metadata_parser;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    MetaDataParseError(#[from] MetaDataParseError),
}

pub fn parse_tsp_instance<P: AsRef<Path>>(instance_path: P) -> Result<TSPInstance, std::io::Error> {
    let mut lines = BufReader::new(File::open(instance_path)?).lines();

    let _ = parse_metadata(&mut lines);

    todo!()
}
