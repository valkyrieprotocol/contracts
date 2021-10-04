use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualifiedContinueOption {
    Eligible,
    Ineligible,
}

impl QualifiedContinueOption {
    pub fn can_participate(&self) -> bool {
        match self {
            QualifiedContinueOption::Eligible => true,
            QualifiedContinueOption::Ineligible => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            QualifiedContinueOption::Eligible => false,
            QualifiedContinueOption::Ineligible => true,
        }
    }
}

impl fmt::Display for QualifiedContinueOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            QualifiedContinueOption::Eligible => f.write_str("eligible"),
            QualifiedContinueOption::Ineligible => f.write_str("ineligible"),
        }
    }
}
