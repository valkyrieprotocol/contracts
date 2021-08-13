use cosmwasm_std::{MessageInfo, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie_qualifier::QualifiedContinueOption;

pub mod instantiate;
pub mod update_admin;
pub mod update_requirement;
pub mod qualify;

mod mock_querier;


const ADMIN: &str = "Admin";

const CONTINUE_OPTION_ON_FAIL: QualifiedContinueOption = QualifiedContinueOption::Ineligible;
const MIN_TOKEN_BALANCE_DENOM_NATIVE: &str = "uluna";
const MIN_TOKEN_BALANCE_AMOUNT: Uint128 = Uint128::new(10000);
const MIN_LUNA_STAKING: Uint128 = Uint128::new(1000);

fn admin_sender() -> MessageInfo {
    mock_info(ADMIN, &[])
}
