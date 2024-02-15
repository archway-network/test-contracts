use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {},
    Reset { count: i32 },
    Register {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
    // DumpState returns the current state
    #[returns(GetDumpStateResponse)]
    DumpState {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}

#[cw_serde]
pub struct GetDumpStateResponse {
    pub count: i32,
    pub owner: String,
    pub connection_id: String,
    pub counterparty_version: String,
}

#[cw_serde]
pub enum SudoMsg  {
    OpenAck {
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
    },
}