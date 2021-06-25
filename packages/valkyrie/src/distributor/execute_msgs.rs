use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub token_contract: String,
    pub booster_config: BoosterConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Spend {
        recipient: String,
        amount: Uint128,
    },
    AddCampaign {
        campaign_addr: String,
        spend_limit: Uint128,
    },
    RemoveCampaign {
        campaign_addr: String,
    },
    UpdateBoosterConfig {
        booster_config: BoosterConfig,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BoosterConfig {
    pub drop_booster_ratio: Decimal,
    pub activity_booster_ratio: Decimal,
    pub plus_booster_ratio: Decimal,
}

impl BoosterConfig {
    pub fn validate(&self) -> StdResult<()> {
        if self.drop_booster_ratio + self.activity_booster_ratio + self.plus_booster_ratio
            != Decimal::one()
        {
            Err(StdError::generic_err("invalid boost_config"))
        } else {
            Ok(())
        }
    }
}
