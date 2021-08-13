use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128, Decimal};
use crate::common::Denom;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub managing_token: String,
    pub terraswap_router: String,
    pub campaign_deposit_fee_burn_ratio: Decimal,
    pub campaign_deposit_fee_recipient: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        admins: Option<Vec<String>>,
        terraswap_router: Option<String>,
        campaign_deposit_fee_burn_ratio: Option<Decimal>,
        campaign_deposit_fee_recipient: Option<String>,
    },
    IncreaseAllowance {
        address: String,
        amount: Uint128,
    },
    DecreaseAllowance {
        address: String,
        amount: Option<Uint128>,
    },
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    Swap {
        denom: Denom,
        amount: Option<Uint128>,
        route: Option<Vec<Denom>>,
    },
    DistributeCampaignDepositFee {
        amount: Option<Uint128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    CampaignDepositFee {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
