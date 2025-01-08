use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{user_info_key, user_info_storage, SaleInfo, State, UserInfo, CONFIG, SALEINFO};

const CONTRACT_NAME: &str = "BANANA_SALE";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const JUNO: &str = "ujuno";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    deps.api.addr_validate(&msg.admin)?;
    deps.api.addr_validate(&msg.token_address)?;

    //presale start, end and claim period validation
    let state = State {
        admin: msg.admin,
        token_address: msg.token_address,
        total_supply: msg.total_supply,
        airdrop_amount: msg.airdrop_amount,
    };
    CONFIG.save(deps.storage, &state)?;

    SALEINFO.save(
        deps.storage,
        &SaleInfo {
            total_aridropped_amount: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::ChangeAdmin { address } => execute_change_admin(deps, env, info, address),
        ExecuteMsg::UpdateConfig { state } => execute_update_config(deps, env, info, state),
        ExecuteMsg::WithdrawTokenByAdmin {} => execute_withdraw_token_by_admin(deps, env, info),
    }
}

fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = CONFIG.load(deps.storage)?;
    let sender = info.sender.to_string();

    //presale start validation check

    //token_amount checking
    let token_amount = state.airdrop_amount;
    let sale_info = SALEINFO.load(deps.storage)?;

    if token_amount + sale_info.total_aridropped_amount > state.total_supply {
        return Err(ContractError::NoEnoughTokens {});
    }

    //sale info update
    SALEINFO.update(deps.storage, |mut sale_info| -> StdResult<_> {
        sale_info.total_aridropped_amount = sale_info.total_aridropped_amount + token_amount;
        Ok(sale_info)
    })?;

    let user_info_key = user_info_key(&sender);

    //user info update
    let user_info = user_info_storage().may_load(deps.storage, user_info_key.clone())?;
    if !user_info.is_none() {
        return Err(ContractError::AlreadyClaimed {});
    }

    user_info_storage().save(
        deps.storage,
        user_info_key,
        &UserInfo {
            address: sender.clone(),
            is_claimed: true,
        },
    )?;

    //messages handling for token transfer and coin send
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let token_transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: sender.clone(),
        amount: token_amount,
    };

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.token_address,
        msg: to_binary(&token_transfer_msg)?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_attribute("claimer", sender)
        .add_messages(messages))
}

//Mint token to this contract
fn execute_change_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    authcheck(deps.as_ref(), &info)?;

    CONFIG.update(deps.storage, |mut state| -> StdResult<_> {
        state.admin = address.clone();
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "change the admin")
        .add_attribute("address", address))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    state: State,
) -> Result<Response, ContractError> {
    authcheck(deps.as_ref(), &info)?;

    CONFIG.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "update configuration"))
}

fn execute_withdraw_token_by_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    authcheck(deps.as_ref(), &info)?;

    let state = CONFIG.load(deps.storage)?;
    let sale_info = SALEINFO.load(deps.storage)?;

    let cw20_transfer_msg = WasmMsg::Execute {
        contract_addr: "token_address".to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: state.admin,
            amount: state.total_supply - sale_info.total_aridropped_amount,
        })?,
        funds: vec![],
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(cw20_transfer_msg);

    Ok(Response::new()
        .add_attribute("action", "withdraw token by admin")
        .add_message(msg))
}

fn authcheck(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let state = CONFIG.load(deps.storage)?;
    if info.sender != state.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn get_coin_info(info: &MessageInfo) -> Result<Coin, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::SeveralCoinsSent {});
    } else {
        let denom = info.funds[0].denom.clone();
        if denom.as_str() != JUNO {
            return Err(ContractError::InvalidCoin {});
        }
        Ok(info.funds[0].clone())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
