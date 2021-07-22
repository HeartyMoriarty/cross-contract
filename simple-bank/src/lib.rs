use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};
use near_sdk::collections::{UnorderedMap, LookupSet};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Bank {
    // (AccountId, AccountId) = (protocolId, userId)
    balances: UnorderedMap<(AccountId, AccountId), f64>,
    owner:  AccountId,
    whitelist: LookupSet<AccountId>
}

impl Default for Bank {
    fn default() -> Self {
        env::panic("bonk".as_bytes())
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

    pub fn get_balance(&self, contract_acc_id: (AccountId, AccountId)) -> f64 {
        self.balances.get(&contract_acc_id).unwrap_or(0.0)
    }

    pub fn deposit(&mut self, acc_id: AccountId, value: f64) {
        self.assert_from_whitelist();
        let contract_acc_id = (env::predecessor_account_id(), acc_id);
        let curr_bal = self.balances.get(&contract_acc_id).unwrap_or(0.0);         
        self.balances.insert(&contract_acc_id, &(curr_bal + value));
    }

    pub fn withdraw(&mut self, acc_id: AccountId, value: f64) {
        self.assert_from_whitelist();
        let contract_acc_id = (env::predecessor_account_id(), acc_id);
        self.assert_has_balance(contract_acc_id.clone(), value);
        let curr_bal = self.balances.get(&contract_acc_id).unwrap_or(0.0);         
        self.balances.insert(&contract_acc_id, &(curr_bal - value));
        // FINISH WITH X CONTRACT CALL
    }
}


impl Bank {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only callable by owner");
    }

    fn assert_has_balance(&self, contract_acc_id: (AccountId, AccountId), value: f64) {
        let balance = self.balances.get(&contract_acc_id).unwrap_or(0.0);
        assert!(balance >= value, "could not withdraw {}, only {} in tha bank for {}", value, balance, &contract_acc_id.1);
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

    fn token() -> String {
        "token".to_string()
    }

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
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
        let contract = Bank::new(alice());
        contract.assert_owner();
    }

    #[test]
    fn add_acc_to_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
    }

    #[test]
    #[should_panic(expected = "only callable by owner")]
    fn add_acc_to_whitelist_as_other_acc() {
        let context = get_context(vec![], false, bob());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        contract.wl_add_acc(bob());
        assert!(!contract.wl_contains(&bob()));

    }

    #[test]
    fn rm_acc_from_whitelist_as_owner() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        contract.wl_add_acc(bob());
        assert!(contract.wl_contains(&bob()));
        contract.wl_remove_acc(bob());
        assert!(!contract.wl_contains(&bob()));
    }

    #[test]
    #[should_panic(expected = "token not whitelisted")]
    fn deposit_from_non_whitelisted_acc() {
        let context = get_context(vec![], false, token());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        let contract_acc_id = (token(), alice());
        contract.deposit(alice(), 100.0);
        assert_eq!(contract.get_balance(contract_acc_id), 0.0);
    }

    #[test]
    fn deposit_from_whitelisted_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        contract.wl_add_acc(token());
        let context = get_context(vec![], false, token());
        testing_env!(context);
        contract.deposit(alice(), 100.0);
        let contract_acc_id = (token(), alice());
        assert_eq!(contract.get_balance(contract_acc_id), 100.0);
    }

    #[test]
    fn withdraw_from_whitelisted_acc() {
        let context = get_context(vec![], false, alice());
        testing_env!(context);
        let mut contract = Bank::new(alice());
        contract.wl_add_acc(token());
        let context = get_context(vec![], false, token());
        testing_env!(context);
        contract.deposit(bob(), 100.0);
        let contract_acc_id = (token(), bob());
        assert_eq!(contract.get_balance(contract_acc_id.clone()), 100.0);
        contract.withdraw(bob(), 50.0);
        assert_eq!(contract.get_balance(contract_acc_id), 50.0);
    }
}
