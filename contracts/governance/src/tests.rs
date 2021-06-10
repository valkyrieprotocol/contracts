use cosmwasm_std::{DepsMut, Env, MessageInfo, Addr, Decimal, Uint128, OwnedDeps, Coin};
use crate::entrypoints;
use valkyrie::governance::messages::{InstantiateMsg, PollConfigInitMsg, ContractConfigInitMsg};
use cosmwasm_std::testing::{MockStorage, MockApi, mock_env, mock_info};
use valkyrie::mock_querier::{WasmMockQuerier, mock_dependencies};

pub const TOKEN_CONTRACT: Addr = Addr::unchecked("TokenContractAddress");

pub const POLL_QUORUM: Decimal = Decimal::percent(30);
pub const POLL_THRESHOLD: Decimal = Decimal::percent(50);
pub const POLL_VOTING_PERIOD: u64 = 10000u64;
pub const POLL_EXECUTION_DELAY_PERIOD: u64 = 10000u64;
pub const POLL_PROPOSAL_DEPOSIT: Uint128 = Uint128::from(10000000000u128);
pub const POLL_SNAPSHOT_PERIOD: u64 = 10u64;

pub fn init(deps: DepsMut, env: Env, info: MessageInfo) {
    let msg = InstantiateMsg {
        contract_config: ContractConfigInitMsg {
            token_contract: TOKEN_CONTRACT.clone(),
        },
        poll_config: PollConfigInitMsg {
            quorum: POLL_QUORUM,
            threshold: POLL_THRESHOLD,
            voting_period: POLL_VOTING_PERIOD,
            execution_delay_period: POLL_EXECUTION_DELAY_PERIOD,
            proposal_deposit: POLL_PROPOSAL_DEPOSIT,
            snapshot_period: POLL_SNAPSHOT_PERIOD,
        }
    };

    entrypoints::instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
}