use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PromiseOrValue, ext_contract, json_types::U128};
use near_sdk::collections::{UnorderedMap, LookupSet};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use serde_json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Message {
    kind: String
}

#[ext_contract(ext_token)]
pub trait FungibleToken {
    fn ft_resolve_transfer(&self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> PromiseOrValue<U128>;
}

pub trait FungibleToken {
    fn ft_resolve_transfer(&self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> PromiseOrValue<U128>;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Bank {
    // (AccountId, AccountId) = (protocolId, userId)
    balances: UnorderedMap<(AccountId, AccountId), u128>,
    owner:  AccountId,
    whitelist: LookupSet<AccountId>
}

impl Default for Bank {
    fn default() -> Self {
        env::panic("bonk".as_bytes())
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Bank {
    // sender = predecessor (token), msg {user, transaction type}
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let log_msg = format!("received {:?} from {} through {} to bank", amount, sender_id, env::predecessor_account_id());
        env::log_str(&log_msg);
        self.assert_from_whitelist();
        let msg: Message = serde_json::from_str(&msg).unwrap();
        let tx_type = &msg.kind;
        let contract_user_id = (env::predecessor_account_id(), sender_id.clone());
        let amount = u128::from(amount);
        match tx_type.as_str() {
            "deposit" => self.deposit(contract_user_id, amount),
            "withdrawal" => self.withdraw(contract_user_id, amount),
            _ => PromiseOrValue::Value(U128(amount))

        }
    }
}

#[near_bindgen]
impl Bank {

    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            balances: UnorderedMap::new(b"b".to_vec()),
            owner,
            whitelist: LookupSet::new(b"w".to_vec())
        }
    }

    // pub fn deploy(&self, account_id: AccountId) {
    //     Promise::new(account_id)
    //         .create_account()
    //         .add_full_access_key(env::signer_account_pk())
    //         .deploy_contract(
    //         );
    // }

    pub fn wl_contains(&mut self, acc_id: &AccountId) -> bool {
        self.whitelist.contains(&acc_id)
    }

    pub fn wl_add_acc(&mut self, acc_id: AccountId) {
        self.assert_owner();
        self.whitelist.insert(&acc_id);
    }
    
    pub fn wl_remove_acc(&mut self, acc_id: AccountId) {
        self.assert_owner();
        self.whitelist.remove(&acc_id);
    }

    pub fn balance_of(&self, contract_id: AccountId, user_id: AccountId) -> U128 {
        let contract_user_id = (contract_id, user_id);
        self.balances.get(&contract_user_id).unwrap_or(0).into()
    }

    fn deposit(&mut self, contract_user_id: (AccountId, AccountId), amount: u128) -> PromiseOrValue<U128> {
        env::log_str("depositing");
        let curr_bal = self.balances.get(&contract_user_id).unwrap_or(0);         
        self.balances.insert(&contract_user_id, &(curr_bal + amount));
        PromiseOrValue::Value(U128(0))
        // PromiseOrValue::Promise(ext_token::ft_resolve_transfer(contract_user_id.1, env::signer_account_id(), U128(amount), &contract_user_id.0, 0, env::prepaid_gas()/4))
    }

    fn withdraw(&mut self, contract_user_id: (AccountId, AccountId), amount: u128) -> PromiseOrValue<U128> {
        env::log_str("withdrawing");
        self.assert_from_whitelist();
        self.assert_has_balance(contract_user_id.clone(), amount);
        let curr_bal = self.balances.get(&contract_user_id).unwrap_or(0);         
        self.balances.insert(&contract_user_id, &(curr_bal - amount));
        PromiseOrValue::Value(U128(0))
        // PromiseOrValue::Promise(ext_token::ft_resolve_transfer(env::signer_account_id(), contract_user_id.1, U128(amount), &contract_user_id.0, 0, env::prepaid_gas()/4))
    }
}


impl Bank {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only callable by owner");
    }

    fn assert_has_balance(&self, contract_user_id: (AccountId, AccountId), amount: u128) {
        let balance = self.balances.get(&contract_user_id).unwrap_or(0);
        assert!(balance >= amount, "{} only has {} tokens", &contract_user_id.1, balance);
    }

    fn assert_from_whitelist(&self) {
        let sender = env::predecessor_account_id();
        assert!(self.whitelist.contains(&sender), "{} not whitelisted", sender);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::accounts;
    use near_sdk::{testing_env, VMContext};

    fn make_transfer_string(kind: String) -> String {
        let mut msg: String = "{\"kind\":\"".to_owned();
        msg.push_str(&kind.to_owned());
        msg.push_str("\"}");
        msg
    }

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool, sender: String) -> VMContext {
        VMContext {
            current_account_id: "bank_hoster.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: sender,
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn owner_set_up() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let contract = Bank::new(accounts(0));
        contract.assert_owner();
    }

    #[test]
    fn add_acc_to_whitelist_as_owner() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(contract.wl_contains(&accounts(1)));
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn add_acc_to_whitelist_as_other_acc() {
        let context = get_context(vec![], false, accounts(1).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(!contract.wl_contains(&accounts(1)));

    }

    #[test]
    fn rm_acc_from_whitelist_as_owner() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(contract.wl_contains(&accounts(1)));
        contract.wl_remove_acc(accounts(1));
        assert!(!contract.wl_contains(&accounts(1)));
    }

    #[test]
    #[should_panic(expected = "charlie not whitelisted")]
    fn deposit_from_non_whitelisted_acc() {
        let context = get_context(vec![], false, accounts(2).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.ft_on_transfer(accounts(0), U128(100), make_transfer_string("deposit".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(0)), U128(0));
    }

    #[test]
    fn deposit_from_whitelisted_acc() {
        // alice (bank) init then whitelists charlie(token)
        // bob sends bank 100 through charlie(token) sends deposit req of 100 to 
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(2));
        let context = get_context(vec![], false, accounts(2).to_string());
        testing_env!(context);
        contract.ft_on_transfer(accounts(1), U128(100), make_transfer_string("deposit".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(1)), U128(100));
    }

    #[test]
    fn withdraw_to_whitelisted_acc() {
        //["alice", "bob", "charlie", "danny", "eugene", "fargo"]
        // alice hosts bank, charlie becomes token and whitelisted to bank,
        // bob sends from charlie(token) 100 deposit
        // (charlie, bob) used to get balance
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(2));
        let context = get_context(vec![], false, accounts(2).to_string());
        testing_env!(context);
        contract.ft_on_transfer(accounts(1), U128(100), make_transfer_string("deposit".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(1)), U128(100));
        contract.ft_on_transfer(accounts(1), U128(50), make_transfer_string("withdrawal".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(1)), U128(50));
    }

    #[test]
    #[should_panic(expected = "bob only has 100 tokens")]
    fn withdraw_too_much() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Bank::new(accounts(0));
        contract.wl_add_acc(accounts(2));
        let context = get_context(vec![], false, accounts(2).to_string());
        testing_env!(context);
        contract.ft_on_transfer(accounts(1), U128(100), make_transfer_string("deposit".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(1)), U128(100));
        contract.ft_on_transfer(accounts(1), U128(200), make_transfer_string("withdrawal".to_owned()));
        assert_eq!(contract.balance_of(accounts(2), accounts(1)), U128(100));
    }
}
