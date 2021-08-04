use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::QualifiedContinueOption;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualificationMsg {
    pub campaign: String,
    pub sender: String,
    pub actor: String,
    pub referrer: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualificationResult {
    pub continue_option: QualifiedContinueOption,
    pub reason: Option<String>,
}
