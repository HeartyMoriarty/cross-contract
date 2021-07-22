use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract};
use near_sdk::collections::{LookupMap, LookupSet};

#[ext_contract(bank)]
pub trait Bank {
    fn deposit(&mut self, acc_id: AccountId, value: f64);
    fn withdraw(&mut self, acc_id: AccountId, value: f64);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    balances:LookupMap<AccountId, f64>,
    owner: AccountId,
    whitelist: LookupSet<AccountId>
}

impl Default for FungibleToken {
    fn default() -> Self {
        env::panic("bonk".as_bytes())
    }
}

#[near_bindgen]
impl FungibleToken {

    #[init]
    pub fn new(owner: AccountId) -> Self {
        let mut balances = LookupMap::new(b"b".to_vec());
        balances.insert(&owner, &0.0);
        let mut whitelist = LookupSet::new(b"w".to_vec());
        whitelist.insert(&owner);
        Self {
            balances,
            owner,
            whitelist
        }
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
        self.balances.insert(&acc_id, &0.0);
    }

    pub fn rm_acc(&mut self, acc_id: AccountId) {
        self.assert_owner();
        self.balances.insert(&acc_id, &0.0);
    }

    pub fn get_balance(&self, acc_id: AccountId) -> f64 {
        self.assert_account_exists(acc_id.clone());
        self.balances.get(&acc_id).unwrap_or(0.0)
    }

    pub fn deposit(&mut self, contract_id: AccountId, value: f64) -> Promise {
        let sender = env::predecessor_account_id();
        bank::deposit(sender, value, &contract_id, 0, env::prepaid_gas()/4)
    }

    pub fn withdraw(&mut self, acc_id: AccountId, value: f64) -> Promise {
        let sender = env::predecessor_account_id();
        bank::withdraw(sender, value, &acc_id, 0, env::prepaid_gas()/4)
    }

    pub fn create_value(&mut self, acc_id: AccountId, value: f64) {
        self.assert_owner();
        self.assert_positive(value);
        let balance = self.balances.get(&acc_id).unwrap_or(0.0);
        self.balances.insert(&acc_id, &(balance + value));
    }

    pub fn add_value(&mut self, acc_id: AccountId, value: f64) {
        self.assert_positive(value);
        self.assert_from_whitelist();
        let balance = self.balances.get(&acc_id).unwrap_or(0.0);
        self.balances.insert(&acc_id, &(balance + value));
    }

    pub fn rm_value(&mut self, acc_id: AccountId, value: f64) {
        self.assert_positive(value);
        self.assert_from_whitelist();
        self.assert_has_balance(acc_id.clone(), value);
        let balance = self.balances.get(&acc_id).unwrap_or(0.0);
        self.balances.insert(&acc_id, &(balance - value));
    }

    pub fn send_value(&mut self, receiver: AccountId, value: f64) {
        let sender = env::predecessor_account_id();
        self.rm_value(sender, value);
        self.add_value(receiver, value);
    }
}

impl FungibleToken {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only callable by owner");
    }

    fn assert_account_exists(&self, acc_id: AccountId) {
        assert!(self.balances.contains_key(&acc_id), "{} account not in token contract", acc_id);
    }
    
    fn assert_has_balance(&self, sender: AccountId, value: f64) {
        let balance = self.balances.get(&sender).unwrap_or(0.0);
        assert!(balance >= value, "{} only has {} tokens", sender, balance);
    }

    fn assert_from_whitelist(&self) {
        let sender = env::predecessor_account_id();
        assert!(self.whitelist.contains(&sender), "{} not whitelisted", sender);
    }

    fn assert_positive(&self, value: f64) {
        assert!(value > 0.0, "Cannot add {}, value must be greater than 0", value);
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
        let contract = FungibleToken::new(alice());
        contract.assert_owner();
    }

    #[test]
    fn add_acc_to_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn add_acc_to_whitelist_as_other_acc() {
        let context = get_context(vec![], false, bob());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.wl_add_acc(bob());
        assert!(!contract.wl_contains(&bob()));

    }

    #[test]
    fn rm_acc_from_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
        contract.wl_remove_acc(bob());
        assert!(!contract.wl_contains(&bob()));
    }

    #[test]
    fn owner_can_add_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_acc(bob());
        contract.assert_account_exists(bob());
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn non_owner_cannot_add_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(bob());
        contract.add_acc(alice());
        assert!(!contract.balances.contains_key(&alice()));
    }

    #[test]
    fn owner_can_create_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_acc(bob());
        contract.assert_account_exists(bob());
    }

    #[test]
    fn owner_can_add_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_acc(bob());
        contract.assert_account_exists(bob());
        contract.add_value(bob(), 100.0);
        assert_eq!(contract.get_balance(bob()), 100.0);
    }

    #[test]
    fn owner_can_rm_value() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_value(bob(), 100.0);
        assert_eq!(contract.get_balance(bob()), 100.0);
        contract.rm_value(bob(), 100.0);
    }

    #[test]
    fn valid_send_between_users() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_value(alice(), 100.0);
        contract.send_value(bob(), 50.0);
        assert_eq!(contract.get_balance(bob()), 50.0);
        assert_eq!(contract.get_balance(alice()), 50.0);
    }

    #[test]
    #[should_panic(expected = "alice only has 0 tokens")]
    fn invalid_send_between_users() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = FungibleToken::new(alice());
        contract.add_value(bob(), 100.0);
        contract.send_value(alice(), 50.0);
        assert_eq!(contract.get_balance(bob()), 100.0);
        assert_eq!(contract.get_balance(alice()), 0.0);
    }
}
