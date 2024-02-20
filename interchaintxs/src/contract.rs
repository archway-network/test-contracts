#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:custodian";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_TIMEOUT_SECONDS: u64 = 60 * 60 * 24 * 7 * 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        connection_id: msg.connection_id.clone(),
        ica_address: "".to_string(),
        voted: false,
        errors: "".to_string(),
        timeout: false,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("connection_id", msg.connection_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register {} => execute::register(deps.as_ref(), env),
        ExecuteMsg::Vote { proposal_id, option, tiny_timeout} => execute::vote(deps.as_ref(), env,  proposal_id, option, tiny_timeout),
    }
}

pub mod execute {
    use cosmwasm_std::CosmosMsg;
    use prost::Message;

    use crate::state::{MsgRegisterInterchainAccount, MsgSubmitTx, MsgVote};

    use super::*;

    pub fn register(deps: Deps, env: Env) -> Result<Response, ContractError> {
        let from_address = env.contract.address.to_string();
        let state = STATE.load(deps.storage)?;
        let connection_id = state.connection_id;
        let interchain_account_id = state.owner.to_string();

        let regsiter_msg = MsgRegisterInterchainAccount {
            from_address: from_address.clone(),
            connection_id: connection_id.clone(),
            interchain_account_id: interchain_account_id.clone(),
        };

        let register_stargate_msg = CosmosMsg::Stargate {
            type_url: "/archway.custodian.v1.MsgRegisterInterchainAccount".to_string(),
            value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
        };

        Ok(Response::new()
            .add_attribute("action", "register")
            .add_attribute("account_owner", from_address)
            .add_attribute("connection_id", connection_id)
            .add_attribute("interchain_account_id", interchain_account_id)
            .add_message(register_stargate_msg))
    }

    pub fn vote(deps: Deps, env: Env, proposal_id: u64, option: i32, tiny_timeout: bool) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let connection_id = state.connection_id;
        let interchain_account_id = state.owner.to_string();
        let from_address = env.contract.address.to_string();
        let ica_address = state.ica_address;

        let vote_msg = MsgVote {
            proposal_id: proposal_id,
            voter: ica_address.clone(),
            option: option,
        };
        
        let vote_msg_stargate_msg = prost_types::Any {
            type_url: "/cosmos.gov.v1.MsgVote".to_string(),
            value: vote_msg.encode_to_vec(),
        };
        let timeout = if tiny_timeout { 1 } else { DEFAULT_TIMEOUT_SECONDS };
        let submittx_msg = MsgSubmitTx {
            from_address: from_address.clone(),
            interchain_account_id: interchain_account_id.clone(),
            connection_id: connection_id.clone(),
            msgs: vec![vote_msg_stargate_msg],
            memo: "sent from contract".to_string(),
            timeout: timeout,
        };
        let submittx_stargate_msg = CosmosMsg::Stargate {
            type_url: "/archway.custodian.v1.MsgSubmitTx".to_string(),
            value: Binary::from(prost::Message::encode_to_vec(&submittx_msg)),
        };
        Ok(Response::new()
            .add_attribute("action", "send")
            .add_message(submittx_stargate_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DumpState {} => to_json_binary(&query::dump_state(deps)?),
    }
}

pub mod query {
    use crate::msg::GetDumpStateResponse;

    use super::*;

    pub fn dump_state(deps: Deps) -> StdResult<GetDumpStateResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetDumpStateResponse {
            owner: state.owner.to_string(),
            connection_id: state.connection_id,
            ica_address: state.ica_address,
            voted: state.voted,
            errors: state.errors,
            timeout: state.timeout,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg { 
        SudoMsg::OpenAck { port_id, channel_id, counterparty_channel_id, counterparty_version, } => sudo::open_ack(
            deps, env, port_id, channel_id, counterparty_channel_id, counterparty_version,),
        SudoMsg::Response { request, data } => sudo::response(deps, request, data),
        SudoMsg::Error { request, details } => sudo::error(deps, request, details),
        SudoMsg::Timeout { request } => sudo::timeout(deps, request),
    }
}

pub mod sudo {
    use crate::msg::{OpenAckVersion, RequestPacket};

    use super::*;

    pub fn open_ack(
        deps: DepsMut,
        _env: Env,
        _port_id: String,
        _channel_id: String,
        _counterparty_channel_id: String,
        counterparty_version: String,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            let open_ack_version: Result<OpenAckVersion, _> =
                serde_json::from_str(&counterparty_version);
            state.ica_address = open_ack_version.unwrap().address.clone();
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "registered ica"))
    }

    pub fn response(deps: DepsMut, _request: RequestPacket, _data: Binary) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.voted = true;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "did action"))
    }

    pub fn error(deps: DepsMut, _request: RequestPacket, details: String) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.errors = details;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "errored"))
    }

    pub fn timeout(deps: DepsMut, _request: RequestPacket) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.timeout = true;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "timeout"))
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::GetDumpStateResponse;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            connection_id: "connection_id".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::DumpState {}).unwrap();
        let value: GetDumpStateResponse = from_json(&res).unwrap();
        assert_eq!("connection_id".to_string(), value.connection_id);
        assert_eq!("creator", value.owner);
        assert_eq!(false, value.voted);
        assert_eq!("", value.ica_address);
    }
}
