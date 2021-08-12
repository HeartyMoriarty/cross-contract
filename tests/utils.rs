use near_sdk_sim::{
    deploy, init_simulator, to_yocto, call, view,
    ContractAccount, UserAccount
  };
use near_sdk::{ json_types::{U128}, AccountId};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "res/token_contract.wasm",
    BANK_WASM_BYTES => "res/simple_bank.wasm",
}

use token_contract::TokenContract;
use simple_bank::BankContract;

const TOKEN_ID: &str = "token";
const BANK_ID: &str = "bank";

pub fn init() -> (UserAccount, ContractAccount<BankContract>, ContractAccount<TokenContract>, UserAccount) {
    let root = init_simulator(None);

    let token = deploy!(
        contract: TokenContract,
        contract_id: TOKEN_ID,
        bytes: &TOKEN_WASM_BYTES,
        signer_account: root,
        init_method: new(root.account_id())
    );

    let bank = deploy!(
        contract: BankContract,
        contract_id: BANK_ID,
        bytes: &BANK_WASM_BYTES,
        signer_account: root,
        init_method: new(root.account_id())
    );

    let alice = root.create_user(AccountId::new_unchecked("alice".to_string()), to_yocto("100000"));

    let res = call!(
        root,
        token.wl_add_acc(bank.account_id())
    );
    println!("{:?}", res);
    res.assert_success();
    let res = call!(
        root,
        bank.wl_add_acc(token.account_id())
    );
    println!("{:?}", res);
    res.assert_success();
    let res = call!(
        root,
        token.add_acc(alice.account_id())
    );
    println!("{:?}", res);
    res.assert_success();
    let res = call!(
        root,
        token.add_acc(bank.account_id())
    );
    println!("{:?}", res);
    res.assert_success();
    let res = call!(
        root,
        bank.add_acc(alice.account_id())
    );
    println!("{:?}", res);
    res.assert_success();
    let res = call!(
        root,
        bank.add_acc(token.account_id())
    );
    println!("{:?}", res);
    res.assert_success();

    (root, bank, token, alice)
}

#[test]
fn send_deposit() {
    let (root, bank, token, alice) = init();

    call!(
        root,
        token.create_amount(alice.account_id(), U128(100))
    )
    .assert_success();

    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    let res = call!(
        alice,
        token.transfer(bank.account_id(), U128(100)),
        deposit = 1
    );
    println!("{:?}", res);
    res.assert_success();

    let outcome: U128 = view!(token.balance_of(bank.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);
}

#[test]
#[should_panic(expected = "The account doesn't have enough balance")]
fn invalid_send_deposit() {
    let (root, bank, token, alice) = init();

    call!(
        root,
        token.create_amount(alice.account_id(), U128(100))
    )
    .assert_success();

    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    call!(
        alice,
        token.transfer(bank.account_id(), U128(200)),
        deposit = 1
    )
    .assert_success();

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
}

#[test]
fn send_withdrawal() {
    let (root, bank, token, alice) = init();

    call!(
        root,
        token.create_amount(alice.account_id(), U128(100))
    )
    .assert_success();

    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    call!(
        alice,
        token.transfer(bank.account_id(), U128(100)),
        deposit = 1
    )
    .assert_success();

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);

    call!(
        alice,
        bank.transfer(token.account_id(), U128(100)),
        deposit = 1
    )
    .assert_success();

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
}

#[test]
fn invalid_send_withdrawal() {
    let (root, bank, token, alice) = init();

    call!(
        root,
        token.create_amount(alice.account_id(), U128(100))
    )
    .assert_success();

    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    call!(
        alice,
        token.transfer(bank.account_id(), U128(100)),
        deposit = 1
    )
    .assert_success();

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);

    let res = call!(
        alice,
        token.transfer(bank.account_id(), U128(200)),
        deposit = 1
    );

    assert!(!res.is_ok(), "alice only has 100 tokens");

    let outcome: U128 = view!(bank.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 0);
}