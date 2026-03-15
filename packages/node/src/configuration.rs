use serde::Deserialize;

/// A struct containing Luminary configuration, to be loaded from environment variables.
#[derive(Deserialize, Debug)]
pub struct LuminaryConfiguration {
    /// The directory where Luminary will look to find projects.
    pub project_directory: String,

    /// The address and port that the Luminary Node will listen on.
    #[serde(default = "default_address")]
    pub address: String,
}

// Provides a default address for the Luminary Node.
fn default_address() -> String {
    return "0.0.0.0:9000".to_string();
}
