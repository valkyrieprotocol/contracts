use cosmwasm_std::Uint128;
use cw20::{Cw20ReceiveMsg, Denom};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie_qualifier::{QualificationMsg, QualifiedContinueOption};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub continue_option_on_fail: QualifiedContinueOption,
    pub min_token_balances: Vec<(Denom, Uint128)>,
    pub min_luna_staking: Uint128,
    pub collateral: Option<(Denom, Uint128)>,
    pub collateral_lock_period: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Qualify(QualificationMsg),
    UpdateAdmin {
        address: String,
    },
    UpdateRequirement {
        continue_option_on_fail: Option<QualifiedContinueOption>,
        min_token_balances: Option<Vec<(Denom, Uint128)>>,
        min_luna_staking: Option<Uint128>,
        collateral_amount: Option<Uint128>,
        collateral_lock_period: Option<u64>,
    },
    DepositCollateral {},
    WithdrawCollateral {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DepositCollateral {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Qualify(QualificationMsg),
    Requirement {},
    Admin {},
    Collaterals {
        address: String,
    },
}
