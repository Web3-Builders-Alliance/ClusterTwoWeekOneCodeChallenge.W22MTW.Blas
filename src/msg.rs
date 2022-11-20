use cosmwasm_std::{Uint128, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, };

use crate::state::{Message, Like};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg { 
    pub stipend: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddMessage {topic:String, message:String},
    AddMessageWithoutFunds {topic:String, message:String},
    LikeMessage {id: Uint128},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCurrentId {},
    GetAllMessage {},
    GetMessagesByAddr { address:String },
    GetMessagesByTopic { topic:String },
    GetMessagesById { id:Uint128 },
    GetLikesById {id:Uint128},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LikesResponse {
    pub likes: Like,
}
