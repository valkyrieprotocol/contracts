use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::QualificationMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Execute {},
    Qualify(QualificationMsg),
}
