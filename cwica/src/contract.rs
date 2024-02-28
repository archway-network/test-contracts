#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cwica";
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
        ExecuteMsg::Vote {
            proposal_id,
            option,
            tiny_timeout,
        } => execute::vote(deps.as_ref(), env, proposal_id, option, tiny_timeout),
    }
}

pub mod execute {
    use cosmwasm_std::CosmosMsg;
    use prost::Message;

    use crate::state::{MsgRegisterInterchainAccount, MsgSendTx, MsgVote};

    use super::*;

    pub fn register(deps: Deps, env: Env) -> Result<Response, ContractError> {
        let from_address = env.contract.address.to_string();
        let state = STATE.load(deps.storage)?;
        let connection_id = state.connection_id;

        let regsiter_msg = MsgRegisterInterchainAccount {
            from_address: from_address.clone(),
            connection_id: connection_id.clone(),
        };

        let register_stargate_msg = CosmosMsg::Stargate {
            type_url: "/archway.cwica.v1.MsgRegisterInterchainAccount".to_string(),
            value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
        };

        Ok(Response::new()
            .add_attribute("action", "register")
            .add_attribute("account_owner", from_address)
            .add_attribute("connection_id", connection_id)
            .add_message(register_stargate_msg))
    }

    pub fn vote(
        deps: Deps,
        env: Env,
        proposal_id: u64,
        option: i32,
        tiny_timeout: bool,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let connection_id = state.connection_id;
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
        let timeout = if tiny_timeout {
            1
        } else {
            DEFAULT_TIMEOUT_SECONDS
        };
        let submittx_msg = MsgSendTx {
            from_address: from_address.clone(),
            connection_id: connection_id.clone(),
            msgs: vec![vote_msg_stargate_msg],
            memo: "sent from contract".to_string(),
            timeout: timeout,
        };
        let submittx_stargate_msg = CosmosMsg::Stargate {
            type_url: "/archway.cwica.v1.MsgSubmitTx".to_string(),
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
        SudoMsg::Ica { account_registered, tx_executed } => sudo::ica(deps, env, account_registered, tx_executed),
        SudoMsg::Error { module_name, error_code, input_payload, error_message } => sudo::error(deps, env, module_name, error_code, input_payload, error_message),
        //SudoMsg::Error { failure, timeout } => sudo::failure(deps, env, failure, timeout),
    }
}

pub mod sudo {
    use crate::msg::{ICAResponse, OpenAck, OpenAckVersion};

    use super::*;

    pub fn ica(
        deps: DepsMut,
        _env: Env,
        open_ack_option: Option<OpenAck>,
        response_option: Option<ICAResponse>,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if open_ack_option.is_some() {
                let open_ack = open_ack_option.unwrap();
                let open_ack_version: Result<OpenAckVersion, _> =
                    serde_json::from_str(&open_ack.counterparty_version);
                state.ica_address = open_ack_version.unwrap().address.clone();
            } 
            if response_option.is_some() {
                state.voted = true;
            }
            Ok(state)
        })?;
        Ok(Response::new())
    }

    pub fn error(deps: DepsMut, _env: Env, module_name: String, error_code: u32, _payload: String, error_message: String) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if module_name == "cwica" {
                if error_code == 1 { // packet timeout error
                    state.timeout = true;
                }
                if error_code == 2 { // submittx execution error
                    state.errors = error_message;
                }
                else {
                    // unknown error
                }
            }
            Ok(state)
        })?;
        Ok(Response::new())
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

    #[test]
    fn ica() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            connection_id: "connection_id".to_string(),
        };
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let open_ack = crate::msg::OpenAck {
            port_id: "port_id".to_string(),
            channel_id: "channel_id".to_string(),
            counterparty_channel_id: "counterparty_channel_id".to_string(),
            counterparty_version: "{\"version\":\"ics27-1\",\"controller_connection_id\":\"connection-0\",\"host_connection_id\":\"connection-0\",\"address\":\"juno1hy7hr06h0jxtalwdehdkxdcnsscxr70v5fzq26nwvghnk4s0deyqld2dke\",\"encoding\":\"proto3\",\"tx_type\":\"sdk_multi_msg\"}".to_string(),
        };
        let msg = SudoMsg::Ica { account_registered: Some(open_ack), tx_executed: None };
        let res = sudo(deps.as_mut(), mock_env(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::DumpState {}).unwrap();
        let value: GetDumpStateResponse = from_json(&res).unwrap();
        assert_eq!("juno1hy7hr06h0jxtalwdehdkxdcnsscxr70v5fzq26nwvghnk4s0deyqld2dke".to_string(), value.ica_address);
        assert_eq!(false, value.voted);

        let tx_executed = crate::msg::ICAResponse {
            packet: crate::msg::RequestPacket {
                source_channel: Some("source_channel".to_string()),
                source_port: Some("source_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                timeout_height: None,
                timeout_timestamp: Some(0),
                sequence: Some(0),
                data: Some(Binary::from("data".as_bytes())),
            },
            data: Binary::from("data".as_bytes()),
        };
        let msg = SudoMsg::Ica { account_registered: None, tx_executed: Some(tx_executed) };
        let res = sudo(deps.as_mut(), mock_env(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::DumpState {}).unwrap();
        let value: GetDumpStateResponse = from_json(&res).unwrap();
        assert_eq!(true, value.voted);
    }
}
