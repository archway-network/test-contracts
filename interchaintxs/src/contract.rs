#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:interchaintxs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
        connection_id: msg.connection_id.clone(),
        counterparty_version: "".to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string())
        .add_attribute("connection_id", msg.connection_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps),
        ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
        ExecuteMsg::Register {  } => execute::register(deps.as_ref(), env),
    }
}

pub mod execute {
    use cosmwasm_std::CosmosMsg;

    use crate::state::MsgRegisterInterchainAccount;

    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }

    pub fn register(deps: Deps, env: Env) -> Result<Response, ContractError> {
        let from_address = env.contract.address.to_string();
        let state = STATE.load(deps.storage)?;
        let connection_id = state.connection_id;
        let interchain_account_id = state.count.to_string();

        let regsiter_msg = MsgRegisterInterchainAccount{
            from_address: from_address.clone(),
            connection_id: connection_id.clone(),
            interchain_account_id: interchain_account_id.clone(),
        };

        let register_stargate_msg = CosmosMsg::Stargate { 
            type_url: "/archway.interchaintxs.v1.MsgRegisterInterchainAccount".to_string(), 
            value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
        };

        Ok(Response::new()
            .add_attribute("action", "register")
            .add_attribute("account_owner", from_address)
            .add_attribute("connection_id", connection_id)
            .add_attribute("interchain_account_id", interchain_account_id)
            .add_message(register_stargate_msg)
        )
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
        QueryMsg::DumpState {} => to_json_binary(&query::dump_state(deps)?),
    }
}

pub mod query {
    use crate::msg::GetDumpStateResponse;

    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }

    pub fn dump_state(deps: Deps) -> StdResult<GetDumpStateResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetDumpStateResponse {
            count: state.count,
            owner: state.owner.to_string(),
            connection_id: state.connection_id,
            counterparty_version: state.counterparty_version,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::OpenAck {port_id, channel_id, counterparty_channel_id, counterparty_version,} => sudo::open_ack(deps, env, port_id, channel_id, counterparty_channel_id, counterparty_version),
    }
}

pub mod sudo {
    use super::*;

    pub fn open_ack(deps: DepsMut, _env: Env, _port_id: String, _channel_id: String, _counterparty_channel_id: String, counterparty_version: String) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.counterparty_version = counterparty_version;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "registered ica"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17, connection_id: "connection_id".to_string()};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17, connection_id: "connection_id".to_string()};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17, connection_id: "connection_id".to_string()};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
