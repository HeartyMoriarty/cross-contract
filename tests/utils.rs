use near_sdk_sim::{
    deploy, init_simulator, to_yocto, call, view,
    ContractAccount, UserAccount
  };
use near_sdk::{ json_types::{U128}, AccountId};
use near_sdk::serde_json::json;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "res/token_contract.wasm",
    BANK_WASM_BYTES => "res/simple_bank.wasm",
}

use token_contract::TokenContract;
use simple_bank::BankContract;

const TOKEN_ID: &str = "token";
const BANK_ID: &str = "bank";

pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        TOKEN_ID.parse().unwrap(),
        "storage_deposit",
        &json!({
            "account_id": user.account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
}

pub fn init() -> (UserAccount, ContractAccount<BankContract>, ContractAccount<TokenContract>, UserAccount, UserAccount) {
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
        init_method: new(token.account_id())
    );

    let alice = root.create_user(AccountId::new_unchecked("alice".to_string()), to_yocto("100000"));
    let bob = root.create_user(AccountId::new_unchecked("bob".to_string()), to_yocto("100000"));

    register_user(&alice);
    register_user(&bob);

    call!(
        root,
        token.add_acc(alice.account_id())
    );
    call!(
        root,
        token.add_acc(bank.account_id())
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
        token.create_amount(alice.account_id(), U128(100))
    )
    .assert_success();

    let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    assert_eq!(u128::from(outcome), 100);

    let res = call!(
        alice,
        token.transfer(bank.account_id(), U128(100), "deposit".to_owned()),
        deposit = 1
    );
    println!("{:?}", res);
    res.assert_success();

    // let outcome: U128 = view!(token.balance_of(bank.account_id())).unwrap_json();
    // assert_eq!(u128::from(outcome), 100);

    // let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
    // assert_eq!(u128::from(outcome), 100);
    // let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    // assert_eq!(u128::from(outcome), 0);
}

// #[test]
// #[should_panic(expected = "alice only has 100 tokens")]
// fn invalid_send_deposit() {
//     let (root, bank, token, alice, _bob) = init();

//     call!(
//         root,
//         token.create_amount(alice.account_id(), U128(100))
//     )
//     .assert_success();

//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);

//     call!(
//         alice,
//         token.transfer(bank.account_id(), U128(200), "deposit".to_owned())
//     )
//     .assert_success();

//     // let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
//     // assert_eq!(u128::from(outcome), 0);
//     // let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     // assert_eq!(u128::from(outcome), 100);
// }

// #[test]
// fn send_withdrawal() {
//     let (root, bank, token, alice, _bob) = init();

//     call!(
//         root,
//         token.create_amount(alice.account_id(), U128(100))
//     )
//     .assert_success();

//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);

//     call!(
//         alice,
//         token.transfer(bank.account_id(), U128(100), "deposit".to_owned())
//     )
//     .assert_success();

//     let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);
//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 0);

//     call!(
//         alice,
//         token.transfer(bank.account_id(), U128(100), "withdrawal".to_owned())
//     )
//     .assert_success();

//     let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
//     assert_eq!(u128::from(outcome), 0);
//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);
// }

// #[test]
// fn invalid_send_withdrawal() {
//     let (root, bank, token, alice, _bob) = init();

//     call!(
//         root,
//         token.create_amount(alice.account_id(), U128(100))
//     )
//     .assert_success();

//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);

//     call!(
//         alice,
//         token.transfer(bank.account_id(), U128(100), "deposit".to_owned())
//     )
//     .assert_success();

//     let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);
//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 0);

//     let res = call!(
//         alice,
//         token.transfer(bank.account_id(), U128(200), "withdrawal".to_owned())
//     );

//     assert!(!res.is_ok(), "alice only has 100 tokens");

//     let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
//     assert_eq!(u128::from(outcome), 100);
//     let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
//     assert_eq!(u128::from(outcome), 0);
// }