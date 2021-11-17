use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualificationMsg {
    pub campaign: String,
    pub sender: String,
    pub actor: String,
    pub referrer: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualificationResult {
    pub can_participate: bool,
    pub participation_reward_rate: Decimal,
    pub referral_reward_rate: Decimal,
    pub memo: Option<String>,
}

impl QualificationResult {
    pub fn success() -> QualificationResult {
        QualificationResult {
            can_participate: true,
            participation_reward_rate: Decimal::one(),
            referral_reward_rate: Decimal::one(),
            memo: None,
        }
    }

    pub fn error(memo: Option<String>) -> QualificationResult {
        QualificationResult {
            can_participate: false,
            participation_reward_rate: Decimal::zero(),
            referral_reward_rate: Decimal::zero(),
            memo,
        }
    }
}
