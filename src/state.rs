use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Message {
    pub id:Uint128,
    pub owner:Addr,
    pub topic: String,
    pub message: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Like {
    pub id: Uint128,
    pub count: Uint128,
}

pub const CURRENT_ID: Item<u128> = Item::new("current_id");

// Stores the amount the funds required for storing a message or liking a message. Set up at Instantiate
pub const STIPEND: Item<Coin> = Item::new("stipend_key");

pub const MESSAGES: Map<u128, Message> = Map::new("messages");

// Records how which message_id have been liked and how many likes.
pub const MESSAGES_LIKE: Map<u128, Like> = Map::new("love_id");
