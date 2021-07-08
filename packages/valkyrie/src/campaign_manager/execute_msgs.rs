use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, StdError, StdResult, Uint128, Binary};
use crate::common::{Denom, ExecutionMsg};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub contract_config: ContractConfigInitMsg,
    pub campaign_config: CampaignConfigInitMsg,
    pub booster_config: BoosterConfigInitMsg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct ContractConfigInitMsg {
    pub governance: String,
    pub fund_manager: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct CampaignConfigInitMsg {
    pub creation_fee_token: String,
    pub creation_fee_amount: Uint128,
    pub creation_fee_recipient: String,
    pub code_id: u64,
    pub distribution_denom_whitelist: Vec<Denom>,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: String,
    pub deactivate_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct BoosterConfigInitMsg {
    pub booster_token: String,
    pub drop_booster_ratio: Decimal,
    pub activity_booster_ratio: Decimal,
    pub plus_booster_ratio: Decimal,
    pub activity_booster_multiplier: Decimal,
    pub min_participation_count: u64,
}

impl BoosterConfigInitMsg {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateContractConfig {
        governance: Option<String>,
        fund_manager: Option<String>,
    },
    UpdateCampaignConfig {
        creation_fee_token: Option<String>,
        creation_fee_amount: Option<Uint128>,
        creation_fee_recipient: Option<String>,
        code_id: Option<u64>,
        withdraw_fee_rate: Option<Decimal>,
        withdraw_fee_recipient: Option<String>,
        deactivate_period: Option<u64>,
    },
    UpdateBoosterConfig {
        booster_token: Option<String>,
        drop_booster_ratio: Option<Decimal>,
        activity_booster_ratio: Option<Decimal>,
        plus_booster_ratio: Option<Decimal>,
        activity_booster_multiplier: Option<Decimal>,
        min_participation_count: Option<u64>,
    },
    AddDistributionDenom {
        denom: Denom,
    },
    RemoveDistributionDenom {
        denom: Denom,
    },
    BoostCampaign {
        campaign: String,
        amount: Uint128,
    },
    FinishBoosting {
        campaign: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    CreateCampaign {
        config_msg: Binary,
        proxies: Vec<String>,
        executions: Vec<ExecutionMsg>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInstantiateMsg {
    pub governance: String,
    pub campaign_manager: String,
    pub fund_manager: String,
    pub admin: String,
    pub creator: String,
    pub proxies: Vec<String>,
    pub config_msg: Binary,
    pub executions: Vec<ExecutionMsg>,
}
