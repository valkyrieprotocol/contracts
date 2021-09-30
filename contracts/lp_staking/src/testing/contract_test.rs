use crate::entrypoints::{instantiate, query};
use crate::states::Config;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr};
use valkyrie::lp_staking::execute_msgs::InstantiateMsg;
use valkyrie::lp_staking::query_msgs::QueryMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        token: "reward".to_string(),
        pair: "pair".to_string(),
        lp_token: "lp_token".to_string(),
    };

    let info = mock_info("addr", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(
        Config {
            token: Addr::unchecked("reward".to_string()),
            pair: Addr::unchecked("pair".to_string()),
            lp_token: Addr::unchecked("lp_token".to_string())
        },
        config
    );
}
