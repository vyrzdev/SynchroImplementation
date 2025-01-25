use std::string::ToString;
use serde::{Deserialize, Serialize};
use crate::vendors::square::SquareConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) profiling_directory: String,
    pub(crate) vendors: Vec<(String, VendorConfig)>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VendorConfig {
    Square(SquareConfig)
}





