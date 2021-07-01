use cosmwasm_std::{CosmosMsg, Env, MessageInfo, ReplyOn, Response, to_binary, Uint128, WasmMsg, from_binary, Addr};
use cw20::Cw20ExecuteMsg;

use valkyrie::campaign::enumerations::Denom;
use valkyrie::campaign::execute_msgs::InstantiateMsg;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, contract_sender, default_sender, DEFAULT_SENDER, expect_generic_err};

use crate::executions::{create_campaign, REPLY_CREATE_CAMPAIGN};
use crate::states::CreateCampaignContext;
use crate::tests::{CAMPAIGN_CODE_ID, CREATION_FEE_AMOUNT, DISTRIBUTOR, GOVERNANCE, TOKEN_CONTRACT, BURN_CONTRACT};

pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
pub const CAMPAIGN_DESCRIPTION: &str = "CampaignDescription";
pub const CAMPAIGN_URL: &str = "https://campaign.url";
pub const PARAMETER_KEY: &str = "vkr";
pub const DISTRIBUTION_TOKEN: &str = "uusd";
pub const DISTRIBUTION_AMOUNTS: [Uint128; 3] = [Uint128(100), Uint128(80), Uint128(20)];

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
    title: String,
    url: String,
    description: String,
    parameter_key: String,
    distribution_denom: Denom,
    distribution_amounts: Vec<Uint128>,
) -> ContractResult<Response> {
    create_campaign(
        deps.as_mut(),
        env,
        info,
        sender,
        amount,
        title,
        url,
        description,
        parameter_key,
        distribution_denom,
        distribution_amounts,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = contract_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        PARAMETER_KEY.to_string(),
        Denom::Native(DISTRIBUTION_TOKEN.to_string()),
        DISTRIBUTION_AMOUNTS.to_vec(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let (env, _, response) = default(&mut deps);

    let submsg = response.submessages.first().unwrap();
    assert_eq!(submsg.id, REPLY_CREATE_CAMPAIGN);
    assert_eq!(submsg.gas_limit, None);
    assert_eq!(submsg.reply_on, ReplyOn::Success);

    match submsg.msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Instantiate { admin, code_id, msg, send: _, label: _ }) => {
            assert_eq!(admin, Some(GOVERNANCE.to_string()));
            assert_eq!(code_id, CAMPAIGN_CODE_ID);

            let init_msg = from_binary::<InstantiateMsg>(&msg).unwrap();
            assert_eq!(init_msg, InstantiateMsg {
                governance: GOVERNANCE.to_string(),
                distributor: DISTRIBUTOR.to_string(),
                token_contract: TOKEN_CONTRACT.to_string(),
                factory: env.contract.address.to_string(),
                burn_contract: BURN_CONTRACT.to_string(),
                title: CAMPAIGN_TITLE.to_string(),
                description: CAMPAIGN_DESCRIPTION.to_string(),
                url: CAMPAIGN_URL.to_string(),
                parameter_key: PARAMETER_KEY.to_string(),
                distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
                distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
                admin: DEFAULT_SENDER.to_string(),
                creator: DEFAULT_SENDER.to_string(),
            });
        },
        _ => panic!("Invalid msg"),
    }

    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DISTRIBUTOR.to_string(),
                amount: CREATION_FEE_AMOUNT,
            }).unwrap(),
        }),
    ]);

    let context = CreateCampaignContext::load(&deps.storage).unwrap();
    assert_eq!(context, CreateCampaignContext {
        code_id: CAMPAIGN_CODE_ID,
        creator: Addr::unchecked(DEFAULT_SENDER),
    });
}

#[test]
fn failed_insufficient_creation_fee() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT.checked_sub(Uint128(1)).unwrap(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        PARAMETER_KEY.to_string(),
        Denom::Native(DISTRIBUTION_TOKEN.to_string()),
        DISTRIBUTION_AMOUNTS.to_vec(),
    );

    expect_generic_err(
        &result,
        format!("Insufficient creation fee (Fee = {})", CREATION_FEE_AMOUNT).as_str(),
    );
}