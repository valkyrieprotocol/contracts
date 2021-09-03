use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enumerations::VoteOption;

// Models

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteInfoMsg {
    pub voter: String,
    pub option: VoteOption,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionPlan {
    pub start_height: u64,
    pub end_height: u64,
    pub amount: Uint128,
}

impl DistributionPlan {
    pub fn release_amount(&self, height: u64) -> Uint128 {
        if self.start_height > height {
            return Uint128::zero();
        }

        let release_amount = self.amount.multiply_ratio(
            height - self.start_height,
            self.end_height - self.start_height,
        );

        std::cmp::min(release_amount, self.amount)
    }
}
