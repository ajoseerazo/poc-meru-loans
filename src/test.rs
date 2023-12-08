extern crate std;

use crate::{CollateralizedLoanContract, CollateralizedLoanContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

fn initialize_tokens<'a>(
    env: Env,
) -> (
    TokenClient<'a>,
    TokenAdminClient<'a>,
    TokenClient<'a>,
    TokenAdminClient<'a>,
) {
    let token_loan_admin = Address::random(&env);

    let (token_lender_loan, token_lender_loan_admin) =
        create_token_contract(&env, &token_loan_admin);

    let token_collateral_admin = Address::random(&env);

    let (token_lender_collateral, token_lender_collateral_admin) =
        create_token_contract(&env, &token_collateral_admin);

    (
        token_lender_loan,
        token_lender_loan_admin,
        token_lender_collateral,
        token_lender_collateral_admin,
    )
}

#[test]
fn initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let token_loan_admin = Address::random(&env);

    let lender = Address::random(&env);

    let (token_loan, token_loan_admin) = create_token_contract(&env, &token_loan_admin);
    token_loan_admin.mint(&lender, &10000000);

    let token_collateral_admin = Address::random(&env);

    let (token_collateral, _) = create_token_contract(&env, &token_collateral_admin);
    // token_loan_admin.mint(&lender, &10000000);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    client.initialize(
        &token_loan.address,
        &token_collateral.address,
        &collateral_rate,
    );

    assert_eq!(client.get_collateral_token(), token_collateral.address);
    assert_eq!(client.get_loan_token(), token_loan.address);
}

#[test]
fn supply_loan_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let token_loan_admin = Address::random(&env);

    let lender = Address::random(&env);

    let (token_lender_loan, token_lender_loan_admin) =
        create_token_contract(&env, &token_loan_admin);
    token_lender_loan_admin.mint(&lender, &400000);

    let token_collateral_admin = Address::random(&env);

    let (token_lender_collateral, _) = create_token_contract(&env, &token_collateral_admin);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    client.initialize(
        &token_lender_loan.address,
        &token_lender_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 400000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    assert_eq!(token_lender_loan.balance(&contract_id), amount_to_supply);
    assert_eq!(token_lender_loan.balance(&lender), 0);
}

#[test]
fn request_loan() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    token_lend_admin.mint(&lender, &500000);
    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    let amount_to_collateralize: i128 = 100;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 500000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    client.request_loan(&borrower, &amount_to_collateralize);

    assert_eq!(token_to_lend.balance(&contract_id), 100000);
    assert_eq!(token_to_lend.balance(&borrower), 400000);
    assert_eq!(token_collateral.balance(&borrower), 0);
    assert_eq!(token_collateral.balance(&contract_id), 100);
}

#[test]
fn get_debt() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    token_lend_admin.mint(&lender, &500000);
    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    let amount_to_collateralize: i128 = 100;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 500000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    client.request_loan(&borrower, &amount_to_collateralize);

    // let interest_rate = 15000;
    let interest_rate: i128 = 0;

    // let interest_rate = 0;

    let loan_amount = 100 * collateral_rate;

    let total_amount_to_pay = loan_amount + (loan_amount * interest_rate / 1000000);

    assert_eq!(client.get_debt(&borrower), total_amount_to_pay);
}

#[test]
fn get_loan_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    token_lend_admin.mint(&lender, &500000);
    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 500000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    assert_eq!(client.get_loan_amount(&100), 400000);
}

#[test]
fn repay_loan() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    let interest_to_pay = 0;

    token_lend_admin.mint(&lender, &400000);
    token_lend_admin.mint(&borrower, &interest_to_pay);

    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    let amount_to_collateralize: i128 = 100;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 400000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    client.request_loan(&borrower, &amount_to_collateralize);

    // client.deposit_collateral(&borrower, &amount_to_collateralize);

    // let amount_to_lend = amount_to_collateralize * collateral_rate;

    // client.issue_loan(&lender, &amount_to_lend);

    // let interest_rate = 15000;
    let interest_rate: i128 = 0;

    let loan_amount = 100 * collateral_rate;

    let amount_to_pay = loan_amount + (loan_amount * interest_rate / 1000000);

    client.repay_loan(&borrower, &amount_to_pay);

    assert_eq!(token_to_lend.balance(&contract_id), amount_to_pay);
    assert_eq!(token_to_lend.balance(&borrower), 0);
    assert_eq!(token_collateral.balance(&borrower), 100);
    assert_eq!(token_collateral.balance(&contract_id), 0);
    assert_eq!(client.get_borrowers().len(), 0);
}

#[test]
fn request_loan_after_repay() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    let interest_to_pay = 0;

    token_lend_admin.mint(&lender, &400000);
    token_lend_admin.mint(&borrower, &interest_to_pay);

    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    let amount_to_collateralize: i128 = 100;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 400000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    client.request_loan(&borrower, &amount_to_collateralize);

    // client.deposit_collateral(&borrower, &amount_to_collateralize);

    // let amount_to_lend = amount_to_collateralize * collateral_rate;

    // client.issue_loan(&lender, &amount_to_lend);

    // let interest_rate = 15000;
    let interest_rate: i128 = 0;

    let loan_amount = 100 * collateral_rate;

    let amount_to_pay = loan_amount + (loan_amount * interest_rate / 1000000);

    client.repay_loan(&borrower, &amount_to_pay);

    assert_eq!(token_to_lend.balance(&contract_id), amount_to_pay);
    assert_eq!(token_to_lend.balance(&borrower), 0);
    assert_eq!(token_collateral.balance(&borrower), 100);
    assert_eq!(token_collateral.balance(&contract_id), 0);
    assert_eq!(client.get_borrowers().len(), 0);

    client.request_loan(&borrower, &amount_to_collateralize);

    assert_eq!(token_to_lend.balance(&contract_id), 0);
    assert_eq!(token_to_lend.balance(&borrower), 400000);
    assert_eq!(token_collateral.balance(&borrower), 0);
    assert_eq!(token_collateral.balance(&contract_id), 100);
    assert_eq!(client.get_borrowers().len(), 1);
}

#[test]
fn get_borrowers() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    let interest_to_pay = 0;

    token_lend_admin.mint(&lender, &400000);
    token_lend_admin.mint(&borrower, &interest_to_pay);

    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    let amount_to_collateralize: i128 = 100;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 400000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    client.request_loan(&borrower, &amount_to_collateralize);

    // client.deposit_collateral(&borrower, &amount_to_collateralize);

    // let amount_to_lend = amount_to_collateralize * collateral_rate;

    // client.issue_loan(&lender, &amount_to_lend);

    // let interest_rate = 15000;

    assert_eq!(client.get_borrowers().first().unwrap().address, borrower);
}

#[test]
fn get_total_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CollateralizedLoanContract);

    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    let (token_to_lend, token_lend_admin, token_collateral, token_collateral_admin) =
        initialize_tokens(env.clone());

    let interest_to_pay = 0;

    token_lend_admin.mint(&lender, &400000);
    token_lend_admin.mint(&borrower, &interest_to_pay);

    token_collateral_admin.mint(&borrower, &100);

    let client = CollateralizedLoanContractClient::new(&env, &contract_id);

    let collateral_rate: i128 = 4000;

    client.initialize(
        &token_to_lend.address,
        &token_collateral.address,
        &collateral_rate,
    );

    let amount_to_supply = 400000;

    client.supply_loan_tokens(&lender, &amount_to_supply);

    assert_eq!(client.get_total_supply(), 400000);
}
