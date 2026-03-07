use serde::Deserialize;

/// A struct containing Luminary configuration, to be loaded from environment variables.
#[derive(Deserialize, Debug)]
pub struct LuminaryConfiguration {
    pub project_directory: String,
}
