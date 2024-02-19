use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
    pub connection_id: String,
    pub counterparty_version: String,
}

pub const STATE: Item<State> = Item::new("state");

/// MsgRegisterInterchainAccount is used to register an account on a remote zone.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterInterchainAccount {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub interchain_account_id: ::prost::alloc::string::String,
}

/// MsgSubmitTx defines the payload for Msg/SubmitTx
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitTx {
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    /// interchain_account_id is supposed to be the unique identifier, e.g.,
    /// lido/kava. This allows contracts to have more than one interchain accounts
    /// on remote zone This identifier will be a part of the portID that we'll
    /// claim our capability for.
    #[prost(string, tag = "2")]
    pub interchain_account_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub msgs: ::prost::alloc::vec::Vec<::prost_types::Any>,
    #[prost(string, tag = "5")]
    pub memo: ::prost::alloc::string::String,
    /// timeout in seconds after which the packet times out
    #[prost(uint64, tag = "6")]
    pub timeout: u64,
}