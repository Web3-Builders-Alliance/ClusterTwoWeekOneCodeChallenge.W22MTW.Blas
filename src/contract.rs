#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128,
};
//use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MessagesResponse, QueryMsg, LikesResponse};
use crate::state::{Like, Message, CURRENT_ID, MESSAGES, MESSAGES_LIKE, STIPEND};

// version info for migration info
//const CONTRACT_NAME: &str = "crates.io:messages";
//const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CURRENT_ID.save(deps.storage, &Uint128::zero().u128())?;
    let required_coin = msg.stipend;

    // A stipend needs to be provided by user. Named coin, greater than 1.
    if required_coin.denom == "" || required_coin.amount < Uint128::from(1u128) {
        return Err(ContractError::ValidCoinRequired {});
    }
    STIPEND.save(deps.storage, &required_coin)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMessage { topic, message } => add_message(deps, info, topic, message),
        ExecuteMsg::AddMessageWithoutFunds { topic, message } => add_message_without_funds(deps, info, topic, message),
        ExecuteMsg::LikeMessage { id } => { like_message( deps, info, id) }
    }
}

pub fn add_message(
    deps: DepsMut,
    info: MessageInfo,
    topic: String,
    message: String,
) -> Result<Response, ContractError> {
    //load current id
    let mut current_id = CURRENT_ID.load(deps.storage)?;
    
    // Making sure the user has sent the funds to create the message
    let stipend = STIPEND.load(deps.storage)?;
    // Need to ask in which order are the comparisons evaluated
    if info.funds.len() != 1
        || info.funds[0].denom != stipend.denom
        || info.funds[0].amount != stipend.amount
    {
        return Err(ContractError::InvalidFundsMessage {
            val1: stipend.denom,
            val2: stipend.amount.to_string(),
        });
    }
 
    //create new message
    let new_message = Message {
        id: Uint128::from(current_id),
        owner: info.sender,
        topic: topic,
        message: message,
    };

    //increment current id
    current_id = current_id.checked_add(1).unwrap();

    MESSAGES.save(deps.storage, new_message.id.u128(), &new_message)?;

    //save current id
    CURRENT_ID.save(deps.storage, &current_id)?;

    Ok(Response::new()
        .add_attribute("action", "add_message")
        .add_attribute("id", new_message.id.to_string()))
}

pub fn add_message_without_funds(
    deps: DepsMut,
    info: MessageInfo,
    topic: String,
    message: String,
) -> Result<Response, ContractError> {
    //load current id
    let mut current_id = CURRENT_ID.load(deps.storage)?;
    
     //create new message
    let new_message = Message {
        id: Uint128::from(current_id),
        owner: info.sender,
        topic: topic,
        message: message,
    };

    //increment current id
    current_id = current_id.checked_add(1).unwrap();

    MESSAGES.save(deps.storage, new_message.id.u128(), &new_message)?;

    //save current id
    CURRENT_ID.save(deps.storage, &current_id)?;

    Ok(Response::new()
        .add_attribute("action", "add_message_without_funds")
        .add_attribute("id", new_message.id.to_string()))
}

pub fn like_message(
    deps: DepsMut,
    info: MessageInfo,
    id: Uint128,
) -> Result<Response, ContractError> {
    // Making sure the user has sent the right funds to like the message
    // QUESTION: in which order the if conditions are evaluated?
    let stipend = STIPEND.load(deps.storage)?;
    if info.funds.len() != 1
        || info.funds[0].denom != stipend.denom
        || info.funds[0].amount != stipend.amount
    {
        return Err(ContractError::InvalidFundsMessage {
            val1: stipend.denom,
            val2: stipend.amount.to_string(),
        });
    }

    // Making sure a message with the id exists. Then get the owner.
    let owner: Addr;
    match MESSAGES.load(deps.storage, id.u128()) {
        Ok(message) => {
            owner = message.owner;
        }
        Err(_) => return Err(ContractError::InvalidMessageID {}),
    }

    // Register the like message
    match MESSAGES_LIKE.load(deps.storage, id.u128()) {
        Ok(mut like) => {
            like.count = like.count.checked_add(Uint128::from(1u128)).unwrap();
            MESSAGES_LIKE.save(deps.storage, id.u128(), &like)?;
        }
        Err(_) => {
            let like = Like {
                id,
                count: Uint128::from(1u128),
            };
            MESSAGES_LIKE.save(deps.storage, id.u128(), &like)?;
        }
    }

    // The received funds will be relayed to the message owner
    let v_stipend = vec![stipend];
    let msg = BankMsg::Send {
        to_address: owner.to_string(),
        amount: v_stipend,
    };

    Ok(Response::new()
        .add_attribute("action", "message_like")
        .add_attribute("message_id", id)
        .add_attribute("sent_to", owner.to_string())
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCurrentId {} => to_binary(&query_current_id(deps)?),
        QueryMsg::GetAllMessage {} => to_binary(&query_all_messages(deps)?),
        QueryMsg::GetMessagesByAddr { address } => {
            to_binary(&query_messages_by_addr(deps, address)?)
        }
        QueryMsg::GetMessagesByTopic { topic } => to_binary(&query_messages_by_topic(deps, topic)?),
        QueryMsg::GetMessagesById { id } => to_binary(&query_messages_by_id(deps, id)?),
        QueryMsg::GetLikesById { id } => to_binary(&query_likes_by_id(deps, id)?),
    }
}

fn query_current_id(deps: Deps) -> StdResult<Uint128> {
    let current_id = CURRENT_ID.load(deps.storage)?;
    Ok(Uint128::from(current_id))
}

fn query_all_messages(deps: Deps) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_addr(deps: Deps, address: String) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .filter(|message| message.owner == address)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_topic(deps: Deps, topic: String) -> StdResult<MessagesResponse> {
    let messages: Vec<Message> = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .filter(|message| message.topic == topic)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_id(deps: Deps, id: Uint128) -> StdResult<MessagesResponse> {
    let message = MESSAGES.load(deps.storage, id.u128())?;
    Ok(MessagesResponse {
        messages: vec![message],
    })
}

fn query_likes_by_id(deps: Deps, id: Uint128) -> StdResult<LikesResponse> {
    let likes = MESSAGES_LIKE.load(deps.storage, id.u128())?;
    Ok(LikesResponse {
        likes
    })
}


#[cfg(test)]
mod tests {
    use crate::error;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, coin, Coin};

    const SENDER: &str = "sender_address";
    const SENDER2: &str = "sender_address2";
    const LIKECOIN_DENOM: &str = "like_coin";
    const LIKECOIN_AMOUNT: u128 = 100;
    const LIKECOIN_WRONG_DENOM: &str = "bad_coin";
    const LIKECOIN_WRONG_AMOUNT: u128 = 50;


    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {stipend: coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)};
        let info = mock_info(SENDER, &[coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    fn add_message(deps: DepsMut, sender: &str, topic: String, message: String) {
        let msg = ExecuteMsg::AddMessage {
            topic: topic,
            message: message,
        };
        let info = mock_info(sender, &[coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)]);
        execute(deps, mock_env(), info, msg).unwrap();
    }

    fn like_message(deps: DepsMut, sender: &str, id: u128, funds: Vec<Coin>) {
        let msg = ExecuteMsg::LikeMessage { id: Uint128::from(id) };
        let info = mock_info(sender, &funds);
        execute(deps, mock_env(), info, msg).unwrap();
    }

    // This function tests an Error scenario
    fn like_message_with_error_response(deps: DepsMut, sender: &str, id: u128, funds: Vec<Coin>) -> Result<Response, ContractError> {
        let msg = ExecuteMsg::LikeMessage { id: Uint128::from(id) };
        let info = mock_info(sender, &funds);
        let resp = execute(deps, mock_env(), info, msg).unwrap_err();
        Err(resp)
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCurrentId {}).unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(Uint128::zero(), value);
    }

    #[test]
    fn add_2_messages_transferring_funds_and_query_all_messages() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        add_message(
            deps.as_mut(),
            SENDER,
            "topic1".to_string(),
            "message1".to_string(),
        );

        add_message(
            deps.as_mut(),
            SENDER,
            "topic2".to_string(),
            "message2".to_string(),
        );

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllMessage {}).unwrap();
        let value: MessagesResponse = from_binary(&res).unwrap();
        assert_eq!(2, value.messages.len());
        assert_eq!(Message{ id: Uint128::zero(), owner: Addr::unchecked(SENDER), topic: "topic1".to_string(), message: "message1".to_string() }, value.messages[0]);
        assert_eq!(Message{ id: Uint128::from(1u128), owner: Addr::unchecked(SENDER), topic: "topic2".to_string(), message: "message2".to_string() }, value.messages[1]);
    }

    #[test]
    fn add_message_like_message() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        add_message(
            deps.as_mut(),
            SENDER,
            "topic1".to_string(),
            "message1".to_string(),
        );

        add_message(
            deps.as_mut(),
            SENDER,
            "topic2".to_string(),
            "message2".to_string(),
        );

        let funds = vec![coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)];
        like_message(deps.as_mut(), SENDER2,0, funds.clone() );
        like_message(deps.as_mut(), SENDER2,0, funds );

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetLikesById { id: Uint128::zero() }).unwrap();
        let value: LikesResponse = from_binary(&res).unwrap();
        
        assert_eq!(Uint128::from(2u128), value.likes.count);
        assert_eq!(Uint128::from(0u128), value.likes.id);

     }


    #[test]
    fn add_message_like_message_with_wrong_funds() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        add_message(
            deps.as_mut(),
            SENDER,
            "topic1".to_string(),
            "message1".to_string(),
        );

        add_message(
            deps.as_mut(),
            SENDER,
            "topic2".to_string(),
            "message2".to_string(),
        );

        let funds = vec![coin(LIKECOIN_WRONG_AMOUNT, LIKECOIN_DENOM)];

        let res : error::ContractError = like_message_with_error_response(deps.as_mut(), SENDER2, 0, funds.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidFundsMessage{val1: LIKECOIN_DENOM.to_string(), val2: LIKECOIN_AMOUNT.to_string()});

    }
  
}

