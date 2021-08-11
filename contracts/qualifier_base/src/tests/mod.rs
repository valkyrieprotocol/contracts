use valkyrie_qualifier::QualifiedContinueOption;
use cosmwasm_std::{Uint128, MessageInfo};
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_admin;
pub mod update_requirement;
pub mod deposit_collateral;
pub mod withdraw_collateral;
pub mod qualify;

mod mock_querier;


const ADMIN: &str = "Admin";

const CONTINUE_OPTION_ON_FAIL: QualifiedContinueOption = QualifiedContinueOption::Ineligible;
const MIN_TOKEN_BALANCE_DENOM_NATIVE: &str = "uluna";
const MIN_TOKEN_BALANCE_AMOUNT: Uint128 = Uint128::new(10000);
const MIN_LUNA_STAKING: Uint128 = Uint128::new(1000);
const COLLATERAL_DENOM_NATIVE: &str = "uusd";
const COLLATERAL_AMOUNT: Uint128 = Uint128::new(100);
const COLLATERAL_LOCK_PERIOD: u64 = 10000;

fn admin_sender() -> MessageInfo {
    mock_info(ADMIN, &[])
}
