use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_contract_standards::fungible_token::{FungibleToken};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, log, near_bindgen, AccountId, PromiseOrValue, json_types::U128};
use near_sdk::collections::{LookupSet};

near_contract_standards::impl_fungible_token_core!(Token, token);
near_contract_standards::impl_fungible_token_storage!(Token, token);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    token: FungibleToken,
    owner: AccountId,
    whitelist: LookupSet<AccountId>,
}

impl Default for Token {
    fn default() -> Self {
        env::panic("bonk".as_bytes())
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Token {
    // sender = user, predecessor = token
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let acc_id = env::predecessor_account_id();
        log!("recieved deposit of {} from {} for {}", u128::from(amount), acc_id, sender_id);
        self.assert_from_whitelist(acc_id.clone());
        self.token.internal_transfer(&acc_id, &sender_id, u128::from(amount), None);
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl Token {

    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            token: FungibleToken::new(b"t".to_vec()),
            owner,
            whitelist: LookupSet::new(b"w".to_vec()),
        }
    }

    pub fn balance_of(&self, acc_id: AccountId) -> U128 {
        U128(self.token.internal_unwrap_balance_of(&acc_id))
    }

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

    pub fn add_acc(&mut self, acc_id: AccountId) {
        self.assert_owner();
        self.token.internal_register_account(&acc_id);
    }

    // TODO: maybe change to ft_transfer_call 
    #[payable]
    pub fn transfer(&mut self, acc_id: AccountId, amount: U128) {
        self.assert_from_whitelist(acc_id.clone());
        self.token.ft_transfer_call(acc_id, amount, None, "nice".to_owned());
    }

    pub fn create_amount(&mut self, acc_id: AccountId, amount: U128) {
        self.assert_owner();
        let balance = self.token.internal_unwrap_balance_of(&acc_id);
        self.token.internal_deposit(&acc_id, balance + u128::from(amount));
    }

    pub fn transfer_internal(&mut self, receiver: AccountId, amount: U128) {
        self.ft_transfer(receiver, amount, None);
    }
}

impl Token {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only callable by owner");
    }

    fn assert_from_whitelist(&self, acc_id: AccountId) {
        assert!(self.whitelist.contains(&acc_id), "{} not whitelisted", acc_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::accounts;
    use near_sdk::{testing_env, VMContext};

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
            attached_deposit: 1,
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
        let contract = Token::new(accounts(0));
        contract.assert_owner();
    }

    #[test]
    fn add_acc_to_whitelist_as_owner() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(contract.wl_contains(&accounts(1)));
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn add_acc_to_whitelist_as_other_acc() {
        let context = get_context(vec![], false, accounts(1).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(!contract.wl_contains(&accounts(1)));

    }

    #[test]
    fn rm_acc_from_whitelist_as_owner() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.wl_add_acc(accounts(1));
        assert!(contract.wl_contains(&accounts(1)));
        contract.wl_remove_acc(accounts(1));
        assert!(!contract.wl_contains(&accounts(1)));
    }

    #[test]
    fn owner_can_add_acc() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.add_acc(accounts(1));
        assert_eq!(u128::from(contract.balance_of(accounts(1))), 0);
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn non_owner_cannot_add_acc() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(1));
        contract.add_acc(accounts(0));
    }

    #[test]
    fn owner_can_create_amount() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.add_acc(accounts(1));
        assert_eq!(u128::from(contract.balance_of(accounts(1))), 0);
    }

    #[test]
    fn valid_send_between_users() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.wl_add_acc(accounts(0));
        contract.add_acc(accounts(0));
        contract.add_acc(accounts(1));
        contract.create_amount(accounts(0), U128(100));
        contract.transfer_internal(accounts(1), U128(50));
        assert_eq!(u128::from(contract.balance_of(accounts(1))), 50);
        assert_eq!(u128::from(contract.balance_of(accounts(0))), 50);
    }

    #[test]
    #[should_panic(expected = "The account doesn't have enough balance")]
    fn invalid_send_between_users() {
        let context = get_context(vec![], false, accounts(0).to_string());
        testing_env!(context);
        let mut contract = Token::new(accounts(0));
        contract.wl_add_acc(accounts(0));
        contract.add_acc(accounts(0));
        contract.add_acc(accounts(1));
        contract.transfer_internal(accounts(1), U128(50));
        assert_eq!(u128::from(contract.balance_of(accounts(1))), 0);
        assert_eq!(u128::from(contract.balance_of(accounts(0))), 50);
    }
}
