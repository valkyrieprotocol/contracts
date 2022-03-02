use cosmwasm_std::{Addr, Decimal, Env, Response};
use cw20::Denom;

use valkyrie::campaign::execute_msgs::MigrateMsg;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{CustomDeps, custom_deps};

use crate::migrations::{migrate, OLD_REWARD_CONFIG, OldRewardConfig};
use valkyrie::test_constants::campaign::{campaign_env, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3, PARTICIPATION_REWARD_LOCK_PERIOD, REFERRAL_REWARD_AMOUNTS, REFERRAL_REWARD_LOCK_PERIOD};
use valkyrie::test_constants::VALKYRIE_TOKEN;
use crate::states::{CampaignState, RewardConfig};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
) -> ContractResult<Response> {
    migrate(deps.as_mut(), env, MigrateMsg {
        participation_reward_distribution_schedule: vec![
            (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.2)),
            (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.2)),
            (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.2)),
        ]
    })
}

pub fn will_success(deps: &mut CustomDeps, chain_id: &str) -> (Env, Response) {
    let mut env = campaign_env();

    env.block.chain_id = chain_id.to_string();

    let response = exec(deps, env.clone()).unwrap();

    (env, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    let (env, info, response) = super::instantiate::default(&mut deps);

    let old = OldRewardConfig{
        participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_lock_period: PARTICIPATION_REWARD_LOCK_PERIOD,
        referral_reward_token: Addr::unchecked(VALKYRIE_TOKEN),
        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        referral_reward_lock_period: REFERRAL_REWARD_LOCK_PERIOD,
    };
    OLD_REWARD_CONFIG.save(deps.as_mut().storage, &old).unwrap();

    will_success(&mut deps, "new-chain-id");

    let reward_config = RewardConfig::load(deps.as_ref().storage).unwrap();
    assert_eq!(reward_config.participation_reward_distribution_schedule, vec![
        (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.2)),
        (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2.2)),
        (PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.0, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.1, Decimal::percent(PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.2)),
    ]);
}
