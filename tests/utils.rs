use near_sdk_sim::{
    deploy, init_simulator, to_yocto, call, view,
    ContractAccount, UserAccount, DEFAULT_GAS
  };
use near_sdk::{ json_types::{U128}};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "res/token_contract.wasm",
    BANK_WASM_BYTES => "res/simple_bank.wasm",
}

use token_contract::TokenContract;
use simple_bank::BankContract;

const TOKEN_ID: &str = "token";
const BANK_ID: &str = "bank";

const STORAGE_AMOUNT: u128 = 10000000000000000000000000000000;

pub fn init() -> (UserAccount, UserAccount, ContractAccount<BankContract>, ContractAccount<TokenContract>, UserAccount, UserAccount) {
    // Use `None` for default genesis configuration; more info below
    let bank_root = init_simulator(None);
    let token_root = init_simulator(None);

    let bank = deploy!(
        contract: BankContract,
        contract_id: BANK_ID,
        bytes: &BANK_WASM_BYTES,
        signer_account: bank_root,
        deposit: STORAGE_AMOUNT,
        gas: DEFAULT_GAS,
        init_method: new(bank_root.account_id())
    );

    let token = deploy!(
        contract: TokenContract,
        contract_id: TOKEN_ID,
        bytes: &TOKEN_WASM_BYTES,
        signer_account: token_root,
        deposit: STORAGE_AMOUNT,
        gas: DEFAULT_GAS,
        init_method: new(token_root.account_id())
    );

    let alice = token_root.create_user(
        "alice".parse().unwrap(),
        to_yocto("200") // initial balance
    );

    let bob = token_root.create_user(
        "bob".parse().unwrap(),
        to_yocto("200") // initial balance
    );

    call!(
        token_root,
        token.add_acc(alice.account_id())
    );
    call!(
        token_root,
        token.add_acc(bank.account_id())
    );
    call!(
        token_root,
        token.wl_add_acc(bank.account_id())
    );
    call!(
        bank_root,
        bank.wl_add_acc(token.account_id())
    );
    call!(
        bank_root,
        token.add_acc(bob.account_id())
    );

    (bank_root, token_root, bank, token, alice, bob)
}

#[test]
fn send_deposit() {
    let (_bank_root, token_root, bank, token, alice, _bob) = init();

    call!(
        token_root,
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

    let outcome: U128 = view!(bank.balance_of((token.account_id(), alice.account_id()))).unwrap_json();
    assert_eq!(u128::from(outcome), 100);
    // let outcome: U128 = view!(token.balance_of(alice.account_id())).unwrap_json();
    // assert_eq!(u128::from(outcome), 0);
}

// #[test]
// #[should_panic(expected = "alice only has 100 tokens")]
// fn invalid_send_deposit() {
//     let (bank_root, token_root, bank, token, alice, _bob) = init();

//     call!(
//         token_root,
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
//     let (bank_root, token_root, bank, token, alice, _bob) = init();

//     call!(
//         token_root,
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
//     let (bank_root, token_root, bank, token, alice, _bob) = init();

//     call!(
//         token_root,
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