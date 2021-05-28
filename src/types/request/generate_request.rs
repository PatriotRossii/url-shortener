use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct GenerateRequest {
    pub link: String,
}
