use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register {},
    Vote {
        proposal_id: u64,
        option: i32,
        tiny_timeout: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // DumpState returns the current state
    #[returns(GetDumpStateResponse)]
    DumpState {},
}

#[cw_serde]
pub struct GetDumpStateResponse {
    pub owner: String,
    pub connection_id: String,
    pub ica_address: String,
    pub voted: bool,
    pub errors: String,
    pub timeout: bool,
}

#[cw_serde]
pub enum SudoMsg  {
    Custodian {
        account_registered: Option<OpenAck>,
        tx_executed: Option<ICAResponse>,
    },
    Error {
        failure: Option<Error>,
        timeout: Option<Timeout>,
    }
}

#[cw_serde]
pub struct OpenAck {
    pub port_id: String,
    pub channel_id: String,
    pub counterparty_channel_id: String,
    pub counterparty_version: String,
}

#[cw_serde]
pub struct ICAResponse {
    pub packet: RequestPacket,
    pub data: Binary,
}

#[cw_serde]
pub struct Error {
    pub packet: RequestPacket,
    pub details: String,
}

#[cw_serde]
pub struct Timeout {
    pub packet: RequestPacket,
}

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_serde]
pub struct RequestPacket {
    pub sequence: Option<u64>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub destination_port: Option<String>,
    pub destination_channel: Option<String>,
    pub data: Option<Binary>,
    pub timeout_height: Option<RequestPacketTimeoutHeight>,
    pub timeout_timestamp: Option<u64>,
}

#[cw_serde]
pub struct RequestPacketTimeoutHeight {
    pub revision_number: Option<u64>,
    pub revision_height: Option<u64>,
}