use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Vertex {
    pub name: String,
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_true")]
    pub is_station: bool,
}

fn default_true() -> bool {
    true
}

impl Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
