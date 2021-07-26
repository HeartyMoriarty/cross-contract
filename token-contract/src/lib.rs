use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, json_types::U128};
use near_sdk::collections::{LookupSet};

#[ext_contract(bank)]
pub trait Bank {
    fn deposit(&mut self, acc_id: AccountId, value: u128);
    fn withdraw(&mut self, acc_id: AccountId, value: u128);
}

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

    pub fn deposit(&mut self, contract_id: AccountId, value: U128) -> Promise {
        let sender = env::predecessor_account_id();
        self.assert_has_balance(sender.clone(), u128::from(value));
        bank::deposit(sender, u128::from(value), &contract_id, 0, env::prepaid_gas()/4)
    }

    pub fn withdraw(&mut self, acc_id: AccountId, value: U128) -> Promise {
        let sender = env::predecessor_account_id();
        bank::withdraw(sender, u128::from(value), &acc_id, 0, env::prepaid_gas()/4)
    }

    pub fn create_value(&mut self, acc_id: AccountId, value: U128) {
        self.assert_owner();
        let balance = self.token.internal_unwrap_balance_of(&acc_id);
        self.token.internal_deposit(&acc_id, balance + u128::from(value));
    }

    pub fn add_value(&mut self, acc_id: AccountId, value: u128) {
        self.assert_from_whitelist();
        self.token.internal_deposit(&acc_id, value);
    }

    pub fn rm_value(&mut self, acc_id: AccountId, value: u128) {
        self.assert_from_whitelist();
        self.token.internal_withdraw(&acc_id, value);
    }

    pub fn transfer(&mut self, receiver: AccountId, value: u128) {
        self.token.internal_transfer(&env::predecessor_account_id(),
            &receiver, value, None);
    }
}

impl Token {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only callable by owner");
    }

    fn assert_has_balance(&self, acc_id: AccountId, value: u128) {
        let balance = self.token.internal_unwrap_balance_of(&acc_id);
        assert!(balance >= value, "{} only has {} tokens", &acc_id, balance);
    }

    fn assert_from_whitelist(&self) {
        let sender = env::predecessor_account_id();
        assert!(self.whitelist.contains(&sender), "{} not whitelisted", sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> String {
        "alice".to_string()
    }

    fn bob() -> String {
        "bob".to_string()
    }

    fn get_context(input: Vec<u8>, is_view: bool, sender: AccountId) -> VMContext {
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
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let contract = Token::new(alice());
        contract.assert_owner();
    }

    #[test]
    fn add_acc_to_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn add_acc_to_whitelist_as_other_acc() {
        let context = get_context(vec![], false, bob());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(bob());
        assert!(!contract.wl_contains(&bob()));

    }

    #[test]
    fn rm_acc_from_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
        contract.wl_remove_acc(bob());
        assert!(!contract.wl_contains(&bob()));
    }

    #[test]
    fn owner_can_add_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.add_acc(bob());
        assert_eq!(u128::from(contract.balance_of(bob())), 0);
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn non_owner_cannot_add_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(bob());
        contract.add_acc(alice());
    }

    #[test]
    fn owner_can_create_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.add_acc(bob());
        assert_eq!(u128::from(contract.balance_of(bob())), 0);
    }

    #[test]
    fn whitelisted_account_can_add_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(alice());
        contract.add_acc(bob());
        assert_eq!(u128::from(contract.balance_of(bob())), 0);
        contract.add_value(bob(), 100);
        assert_eq!(u128::from(contract.balance_of(bob())), 100);
    }

    #[test]
    fn whitelisted_account_can_rm_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(alice());
        contract.add_acc(bob());
        contract.add_value(bob(), 100);
        assert_eq!(u128::from(contract.balance_of(bob())), 100);
        contract.rm_value(bob(), 100);
    }

    #[test]
    fn valid_send_between_users() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(alice());
        contract.add_acc(alice());
        contract.add_acc(bob());
        contract.add_value(alice(), 100);
        contract.transfer(bob(), 50);
        assert_eq!(u128::from(contract.balance_of(bob())), 50);
        assert_eq!(u128::from(contract.balance_of(alice())), 50);
    }

    #[test]
    #[should_panic(expected = "The account doesn't have enough balance")]
    fn invalid_send_between_users() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Token::new(alice());
        contract.wl_add_acc(alice());
        contract.add_acc(alice());
        contract.add_acc(bob());
        contract.transfer(bob(), 50);
        assert_eq!(u128::from(contract.balance_of(bob())), 0);
        assert_eq!(u128::from(contract.balance_of(alice())), 50);
    }
}
