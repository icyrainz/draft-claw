use std::collections::HashSet;
use std::io::prelude::*;
use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CardRating {
    pub format: String,
    pub name: String,
    pub rating: String,
}
