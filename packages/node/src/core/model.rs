use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct LuminaryProject {
    pub name: String,
    pub dir: String,
    pub services: HashMap<String, LuminaryService>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct LuminaryService {
    pub name: String,
}
