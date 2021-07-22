use near_sdk_sim::{
    deploy, init_simulator, to_yocto, call, view,
    ContractAccount, UserAccount, DEFAULT_GAS
  };

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "res/token_contract.wasm",
    BANK_WASM_BYTES => "res/simple_bank.wasm",
}

use token_contract::FungibleTokenContract;
use simple_bank::BankContract;

const TOKEN_ID: &str = "token";
const BANK_ID: &str = "bank";

const STORAGE_AMOUNT: u128 = 10000000000000000000000000000000;

pub fn init() -> (UserAccount, ContractAccount<BankContract>, ContractAccount<FungibleTokenContract>, UserAccount, UserAccount) {
    // Use `None` for default genesis configuration; more info below
    let root = init_simulator(None);

    let bank = deploy!(
        contract: BankContract,
        contract_id: BANK_ID,
        bytes: &BANK_WASM_BYTES,
        signer_account: root,
        deposit: STORAGE_AMOUNT,
        gas: DEFAULT_GAS,
        init_method: new(root.account_id())
    );

    let token = deploy!(
        contract: FungibleTokenContract,
        contract_id: TOKEN_ID,
        bytes: &TOKEN_WASM_BYTES,
        signer_account: root,
        deposit: STORAGE_AMOUNT,
        gas: DEFAULT_GAS,
        init_method: new(root.account_id())
    );

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("200") // initial balance
    );

    let bob = root.create_user(
        "bob".parse().unwrap(),
        to_yocto("200") // initial balance
    );

    call!(
        root,
        token.add_acc(alice.account_id())
    );
    call!(
        root,
        token.wl_add_acc(bank.account_id())
    );
    call!(
        root,
        bank.wl_add_acc(token.account_id())
    );
    call!(
        root,
        token.add_acc(bob.account_id())
    );

    (root, bank, token, alice, bob)
}

#[test]
fn send_deposit() {
    let (root, bank, token, alice, _bob) = init();

    call!(
        root,
        token.create_value(alice.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);

    call!(
        alice,
        token.deposit(bank.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 100.0);
}

#[test]
#[should_panic(expected = "alice only has 100 tokens")]
fn invalid_send_deposit() {
    let (root, bank, token, alice, _bob) = init();

    call!(
        root,
        token.create_value(alice.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);

    call!(
        alice,
        token.deposit(bank.account_id(), 200.0)
    )
    .assert_success();

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 0.0);
    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);
}

#[test]
fn send_withdrawal() {
    let (root, bank, token, alice, _bob) = init();

    call!(
        root,
        token.create_value(alice.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);

    call!(
        alice,
        token.deposit(bank.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 100.0);
    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 0.0);

    call!(
        alice,
        token.withdraw(bank.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 0.0);
    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);
}

#[test]
fn invalid_send_withdrawal() {
    let (root, bank, token, alice, _bob) = init();

    call!(
        root,
        token.create_value(alice.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 100.0);

    call!(
        alice,
        token.deposit(bank.account_id(), 100.0)
    )
    .assert_success();

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 100.0);
    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 0.0);

    let res = call!(
        alice,
        token.withdraw(bank.account_id(), 200.0)
    );

    assert!(!res.is_ok(), "alice only has 100 tokens");

    let outcome: f64 = view!(bank.get_balance((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(outcome, 100.0);
    let outcome: f64 = view!(token.get_balance(alice.account_id())).unwrap_json();
    assert_eq!(outcome, 0.0);
}