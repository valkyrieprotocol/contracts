use cosmwasm_std::{Addr, Env, MessageInfo, Response};

use valkyrie::common::{ContractResult};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::proxy::execute_msgs::{DexInfo, DexType, InstantiateMsg};

use crate::executions::*;
use crate::states::{Config};
use valkyrie::test_constants::proxy::{ADMIN, ASTRO_FACTORY, proxy_env, proxy_sender};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        astroport_factory: ASTRO_FACTORY.to_string(),
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn will_success(
    deps: &mut CustomDeps,
) -> (Env, MessageInfo, Response) {
    let env = proxy_env();
    let info = proxy_sender(ADMIN);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps
    )
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    let (env, _, _) = default(&mut deps);

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config, Config {
        admin: Addr::unchecked(ADMIN),
        fixed_dex: None,
        dex_list: vec![
            DexInfo {
                dex_type: DexType::Astroport,
                factory: Addr::unchecked(ASTRO_FACTORY),
            }
        ],
    });
}