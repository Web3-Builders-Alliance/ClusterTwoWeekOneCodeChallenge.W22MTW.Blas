#[cfg(test)]
mod tests {
    use crate::ContractError;
    use crate::helpers::MessagesContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MessagesResponse, QueryMsg, LikesResponse};
    use cosmwasm_std::{coin, to_binary, Addr, BankMsg, BankQuery, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_messages() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER1: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const USER2: &str = "juno1and87527ua866yqh2mpakl9zkxzj5myu6f87ll";
    const ADMIN: &str = "juno1and87527ua866yqh2mpakl9zkxzj5myu6f87ld";
    const NATIVE_DENOM: &str = "denom";
    const LIKECOIN_DENOM: &str = "like_coin";
    const LIKECOIN_AMOUNT: u128 = 100;
    const LIKECOIN_WRONG_DENOM: &str = "bad_coin";
    const LIKECOIN_WRONG_AMOUNT: u128 = 200;

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER1),
                    vec![Coin {
                        denom: LIKECOIN_DENOM.to_string(),
                        amount: Uint128::new(1000),
                    }, Coin {
                        denom: LIKECOIN_WRONG_DENOM.to_string(),
                        amount: Uint128::new(500),
                    }],
                )
                .unwrap();
        })
    }

    // fn send_coins_to_user2(app: &mut App) {
    //     app.send_tokens(
    //         Addr::unchecked(USER1),
    //         Addr::unchecked(USER2),
    //         &vec![coin(1000, NATIVE_DENOM)],
    //     );
    // }

    // We get back the blockchain App and the "Message contract" message_id
    fn store_code() -> (App, u64) {
        let mut app = mock_app();
        let messages_id = app.store_code(contract_messages());
        (app, messages_id)
    }

    // What is important here is that the struct keeps the contract address
    // and the methods addr() and call(.. Execute Msg .. ) can be called based on that address.
    // The implementation can be found on helpers.rs
    fn messages_contract(app: &mut App, code_id: u64) -> MessagesContract {
        // At instantiate the stipend that needs to be sent to add a message or like a message is set up.
        let msg = InstantiateMsg {stipend: coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)};
          let messages_contract_address = app
            .instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "messages", None)
            .unwrap();
        MessagesContract(messages_contract_address)
    }

    fn get_all_messages(app: &App, messages_contract: &MessagesContract) -> MessagesResponse {
        app.wrap()
            .query_wasm_smart(messages_contract.addr(), &QueryMsg::GetAllMessage {})
            .unwrap()
    }

    fn get_messages_by_id(app: &App, messages_contract: &MessagesContract, id:Uint128) -> MessagesResponse {
        app.wrap()
            .query_wasm_smart(messages_contract.addr(), &QueryMsg::GetMessagesById { id })
            .unwrap()
    }

    fn get_likes_by_id(app: &App, messages_contract: &MessagesContract, id:Uint128) -> LikesResponse {
        app.wrap()
            .query_wasm_smart(messages_contract.addr(), &QueryMsg::GetLikesById { id })
            .unwrap()
    }

    fn get_balance(app: &App, user: String, denom: String) -> Coin {
        app.wrap().query_balance(user, denom).unwrap()
    }

    fn print_balances(app: &App, extra_info: &str, contract_address: Addr) {
        let addr = USER1.to_string();
        let denom = LIKECOIN_DENOM.to_string();
        let coin = app.wrap().query_balance(addr, denom).unwrap();
        println!("\n\t{} ****User1 Balance {:?}",extra_info,coin);

        let addr = USER2.to_string();
        let denom = LIKECOIN_DENOM.to_string();
        let coin = app.wrap().query_balance(addr, denom).unwrap();
        println!("\t{} ****User2 Balance {:?}",extra_info,coin);

        let addr = contract_address.to_string();
        let denom = LIKECOIN_DENOM.to_string();
        let coin = app.wrap().query_balance(addr, denom).unwrap();
        println!("\t{} ****Contract Balance {:?}",extra_info,coin);
    }

    fn add_message(
        app: &mut App,
        messages_contract: &MessagesContract,
        owner: Addr,
        topic: String,
        message: String,
        funds: Vec<Coin>,
    ) {
        //use ExecuteMsg to add a message
        //use app.execute_contract to send message to contract
        let msg = ExecuteMsg::AddMessage { topic, message };
        // print_balances(app, "Before Addding Message", messages_contract.addr());
         let res = app.execute_contract(owner, messages_contract.addr(), &msg, &funds)
            .unwrap();
        print_balances(app, "After Addding Message", messages_contract.addr());
    }

    fn add_message_without_funds_requirement(
        app: &mut App,
        messages_contract: &MessagesContract,
        owner: Addr,
        topic: String,
        message: String,
    ) {
        //use ExecuteMsg to add a message
        //use app.execute_contract to send message to contract
        let msg = ExecuteMsg::AddMessageWithoutFunds { topic, message };
        // print_balances(app, "Before Addding Message", messages_contract.addr());
         let res = app.execute_contract(owner, messages_contract.addr(), &msg, &[])
            .unwrap();
        print_balances(app, "After Addding Message without Funds requirements", messages_contract.addr());
    }

    fn add_message_wrong_funds(
        app: &mut App,
        messages_contract: &MessagesContract,
        owner: Addr,
        topic: String,
        message: String,
        funds: Vec<Coin>,
    ) {
        //use ExecuteMsg to add a message
        //use app.execute_contract to send message to contract
        let msg = ExecuteMsg::AddMessage { topic, message } ;
        print_balances(app, "Before Addding Message with wrong funds", messages_contract.addr());
         let res= app.execute_contract(owner, messages_contract.addr(), &msg, &funds)
            .unwrap_err();
        println!("\t\t{:?}", res);
        // It would be interesting to see how to assert an Error type. 
        print_balances(app, "After Addding Message with wrong funds", messages_contract.addr());
    }


    fn like_message(
        app: &mut App,
        messages_contract: &MessagesContract,
        owner: Addr,
        id: u128,
        funds: Vec<Coin>,
    ) {
        //use ExecuteMsg to add a message
        //use app.execute_contract to send message to contract
        let msg = ExecuteMsg::LikeMessage { id: Uint128::from(id) };
        let res = app.execute_contract(owner, messages_contract.addr(), &msg, &funds)
            .unwrap();
        
        print_balances(app, "After Liking Message", messages_contract.addr());
    }

    #[test]   
    fn add_two_messages_query_all_messages() {
        let (mut app, code_id) = store_code();
        let messages_contract = messages_contract(&mut app, code_id);
        let funds = vec![coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)];
        add_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic1".to_string(), "message1".to_string(), funds.clone());
        add_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic2".to_string(), "message2".to_string(), funds.clone());
        // add_message(&mut app, &messages_contract, Addr::unchecked(USER2.to_string()), "topic1".to_string(), "message2".to_string(), funds);
        let message_response = get_all_messages(&app, &messages_contract);
        assert_eq!(message_response.messages.len(),2);
    }

    #[test]
    fn add_one_message_sending_wrong_funds_and_query_all_messages() {
        let (mut app, code_id) = store_code();
        let messages_contract = messages_contract(&mut app, code_id);
        let funds = vec![coin(LIKECOIN_WRONG_AMOUNT, LIKECOIN_DENOM)];
        add_message_wrong_funds(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic1".to_string(), "message1".to_string(), funds.clone());
        let message_response = get_all_messages(&app, &messages_contract);
        assert_eq!(message_response.messages.len(),0);
    }  
    
    #[test]
    fn add_two_messages_sending_like_one_and_query_like_messages() {
        let (mut app, code_id) = store_code();
        let messages_contract = messages_contract(&mut app, code_id);
        let funds = vec![coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)];
        add_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic1".to_string(), "message1".to_string(), funds.clone());
        add_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic2".to_string(), "message2".to_string(), funds.clone());
        like_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), 1, funds.clone());
 
        let message_response = get_all_messages(&app, &messages_contract);
        let like_response = get_likes_by_id(&app, &messages_contract,Uint128::from(1u128));
        assert_eq!(message_response.messages.len(),2);
        assert_eq!(like_response.likes.count,Uint128::from(1u128));
    }  


    #[test]
    fn add_two_messages_from_two_users_sending_like_one_and_query_like_messages() {
        // In order to test an scenario where a second user creates a message the add_message_without_funds functionality has been implemented in the contract.
        // When creating the app, only one initial_balance can be set, so, as to wallets can not be set up based on my current knowledge
        // the best way to have that user with a message created, it is to allow him to create the message without funds.
        //
        let (mut app, code_id) = store_code();
        let messages_contract = messages_contract(&mut app, code_id);
        let funds = vec![coin(LIKECOIN_AMOUNT, LIKECOIN_DENOM)];
        add_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), "topic1".to_string(), "message1".to_string(), funds.clone());
        add_message_without_funds_requirement(&mut app, &messages_contract, Addr::unchecked(USER2.to_string()), "topic2".to_string(), "message2".to_string());
        like_message(&mut app, &messages_contract, Addr::unchecked(USER1.to_string()), 1, funds.clone());
 
        let message_response = get_all_messages(&app, &messages_contract);
        let like_response = get_likes_by_id(&app, &messages_contract,Uint128::from(1u128));
        assert_eq!(message_response.messages.len(),2);
        assert_eq!(like_response.likes.count,Uint128::from(1u128));
    }  


}
