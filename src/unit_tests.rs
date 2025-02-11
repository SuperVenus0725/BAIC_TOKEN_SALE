#[cfg(test)]
use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::{query_get_user_infos, query_sale_info, query_user_info};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

fn setup_contract(deps: DepsMut, env: Env) {
    let instantiate_msg = InstantiateMsg {
        admin: "admin".to_string(),
        token_address: "token_address".to_string(),
        total_supply: Uint128::new(10000),
        airdrop_amount: Uint128::new(100),
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps, mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn init_contract() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        admin: "admin".to_string(),
        token_address: "token_address".to_string(),
        total_supply: Uint128::new(10000),
        airdrop_amount: Uint128::new(100),
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_buy() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    setup_contract(deps.as_mut(), env.clone());

    env.block.time = env.block.time.plus_seconds(300);

    let info = mock_info(
        "user1",
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::new(100),
        }],
    );
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token_address".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "user1".to_string(),
                amount: Uint128::new(100)
            })
            .unwrap(),
            funds: vec![]
        })
    );
}
