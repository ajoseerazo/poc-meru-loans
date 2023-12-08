#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

#[contract]
pub struct CollateralizedLoanContract;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CollateralAmount,
    Admin,
    LoanToken,
    CollateralToken,
    InterestRate,
    CollateralRate,
    Lender,
    Borrower,
    Debt,
    Installments,
    TotalSupply,
    Borrowers,
    BorrowersAddresses,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Borrower {
    pub address: Address,
    pub amount_locked: i128,
    pub debt: i128,
}

#[contractimpl]
impl CollateralizedLoanContract {
    pub fn has_administrator(e: Env) -> bool {
        let key = DataKey::Admin;
        e.storage().instance().has(&key)
    }

    pub fn read_administrator(e: Env) -> Address {
        let key = DataKey::Admin;
        e.storage().instance().get(&key).unwrap()
    }

    pub fn write_administrator(e: Env, id: Address) {
        let key = DataKey::Admin;
        e.storage().instance().set(&key, &id);
    }

    pub fn initialize(
        env: Env,
        loan_token: Address,
        collateral_token: Address,
        collateral_rate: i128,
    ) {
        env.storage()
            .instance()
            .set(&DataKey::LoanToken, &loan_token);

        env.storage()
            .instance()
            .set(&DataKey::CollateralToken, &collateral_token);

        let interest_rate: i128 = 15000; // TODO: Should be configurated through params

        env.storage()
            .instance()
            .set(&DataKey::InterestRate, &interest_rate);

        env.storage()
            .instance()
            .set(&DataKey::CollateralRate, &collateral_rate);

        let installments = 3;

        env.storage()
            .instance()
            .set(&DataKey::Installments, &installments);
    }

    pub fn request_loan(env: Env, from: Address, amount: i128) -> i128 {
        // Perhaps this check should be enabled...
        if amount == 0 {
            panic!("deposit amount must not be zero");
        }
        // Make sure `from` address authorized the deposit call with all the
        // arguments.
        from.require_auth();

        let collateral_token = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();

        let collateral_rate = env
            .storage()
            .instance()
            .get(&DataKey::CollateralRate)
            .unwrap_or(0);

        let mut total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        let debt = amount * collateral_rate;

        if total_supply < debt {
            panic!("total supply is not enough");
        }

        // Transfer token from `from` to this contract address.
        token::Client::new(&env, &collateral_token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );

        let mut borrowers: Vec<Borrower> = env
            .storage()
            .instance()
            .get(&DataKey::Borrowers)
            .unwrap_or(Vec::new(&env));

        let mut borrower_addresses: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::BorrowersAddresses)
            .unwrap_or(Vec::new(&env));

        borrower_addresses.push_back(from.clone());

        env.storage()
            .instance()
            .set(&DataKey::BorrowersAddresses, &borrower_addresses);

        let loan_token = env.storage().instance().get(&DataKey::LoanToken).unwrap();

        total_supply -= debt;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &total_supply);

        token::Client::new(&env, &loan_token).transfer(
            &env.current_contract_address(),
            &from,
            &debt,
        );

        let borrower = Borrower {
            address: from,
            amount_locked: amount,
            debt: debt, // paytime: paytime,
        };

        borrowers.push_back(borrower);

        env.storage()
            .instance()
            .set(&DataKey::Borrowers, &borrowers);

        env.storage().instance().bump(100, 100);

        debt
    }

    pub fn supply_loan_tokens(env: Env, from: Address, amount_to_lend: i128) {
        if amount_to_lend == 0 {
            panic!("loan amount must not be zero");
        }

        from.require_auth();

        let loan_token = env.storage().instance().get(&DataKey::LoanToken).unwrap();

        token::Client::new(&env, &loan_token).transfer(
            &from,
            &env.current_contract_address(),
            &amount_to_lend,
        );

        // env.storage().instance().set(&DataKey::Lender, &from);
        let mut total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        total_supply += amount_to_lend;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &total_supply);

        env.storage().instance().bump(100, 100);
    }

    pub fn get_loan_amount(env: Env, collateral_amount: i128) -> i128 {
        let collateral_rate = env
            .storage()
            .instance()
            .get(&DataKey::CollateralRate)
            .unwrap_or(0);

        collateral_amount * collateral_rate
    }

    // pub fn make_monthly_payment(env: Env, amount: i128) {
    //     let collateral_amount: i128 = env
    //         .storage()
    //         .instance()
    //         .get(&DataKey::CollateralAmount)
    //         .unwrap_or(0);

    //     let collateral_rate = env
    //         .storage()
    //         .instance()
    //         .get(&DataKey::CollateralRate)
    //         .unwrap_or(0);

    //     let installments = env
    //         .storage()
    //         .instance()
    //         .get(&DataKey::Installments)
    //         .unwrap_or(1);

    //     let loan_amount = collateral_amount * collateral_rate;

    //     let monthly_amount = loan_amount / installments;

    //     let interest_rate: i128 = env
    //         .storage()
    //         .instance()
    //         .get(&DataKey::InterestRate)
    //         .unwrap();

    //     let monthly_amount_to_pay = monthly_amount + (loan_amount * interest_rate / 1000000);

    //     if amount < monthly_amount_to_pay {
    //         panic!("monehtly payment amount is less than allowed")
    //     }

    //     let mut debt: i128 = env.storage().instance().get(&DataKey::Debt).unwrap();

    //     debt = debt - amount;

    //     if debt < 0 {
    //         debt = 0;
    //     }

    //     let loan_token = env.storage().instance().get(&DataKey::LoanToken).unwrap();

    //     let borrower: Address = env.storage().instance().get(&DataKey::Borrower).unwrap();

    //     let lender: Address = env.storage().instance().get(&DataKey::Lender).unwrap();

    //     token::Client::new(&env, &loan_token).transfer(&borrower, &lender, &amount);

    //     env.storage().instance().set(&DataKey::Debt, &debt);

    //     if debt <= 0 {
    //         let collateral_token = env
    //             .storage()
    //             .instance()
    //             .get(&DataKey::CollateralToken)
    //             .unwrap();

    //         token::Client::new(&env, &collateral_token).transfer(
    //             &env.current_contract_address(),
    //             &borrower,
    //             &collateral_amount,
    //         );
    //     }
    // }

    pub fn repay_loan(env: Env, from: Address, amount: i128) -> i128 {
        let mut borrowers: Vec<Borrower> = env
            .storage()
            .instance()
            .get(&DataKey::Borrowers)
            .unwrap_or(Vec::new(&env));

        let mut borrowers_addresses: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::BorrowersAddresses)
            .unwrap_or(Vec::new(&env));

        let index = borrowers_addresses.first_index_of(from).unwrap();

        let borrower = borrowers.get(index).unwrap();

        let loan_amount = borrower.debt;

        let interest_rate: i128 = env
            .storage()
            .instance()
            .get(&DataKey::InterestRate)
            .unwrap();

        let total_amount_to_pay = loan_amount + (loan_amount * interest_rate / 1000000);

        if amount != total_amount_to_pay {
            panic!("payment amount is not correct");
        }

        // let lender: Address = env.storage().instance().get(&DataKey::Lender).unwrap();

        borrower.address.require_auth();

        let loan_token = env.storage().instance().get(&DataKey::LoanToken).unwrap();

        token::Client::new(&env, &loan_token).transfer(
            &borrower.address,
            &env.current_contract_address(),
            &total_amount_to_pay,
        );

        let collateral_token = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();

        token::Client::new(&env, &collateral_token).transfer(
            &env.current_contract_address(),
            &borrower.address,
            &borrower.amount_locked,
        );

        borrowers_addresses.remove(index);
        borrowers.remove(index);

        env.storage()
            .instance()
            .set(&DataKey::Borrowers, &borrowers);

        env.storage()
            .instance()
            .set(&DataKey::BorrowersAddresses, &borrowers_addresses);

        let mut total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        total_supply += total_amount_to_pay;

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &total_supply);

        env.storage().instance().bump(100, 100);

        0
    }

    pub fn get_debt(env: Env, from: Address) -> i128 {
        let borrowers: Vec<Borrower> = env
            .storage()
            .instance()
            .get(&DataKey::Borrowers)
            .unwrap_or(Vec::new(&env));

        let borrowers_addresses: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::BorrowersAddresses)
            .unwrap_or(Vec::new(&env));

        let index = borrowers_addresses.first_index_of(from).unwrap();

        let borrower = borrowers.get(index).unwrap();

        let loan_amount = borrower.debt;

        let interest_rate: i128 = env
            .storage()
            .instance()
            .get(&DataKey::InterestRate)
            .unwrap();

        let total_amount_to_pay = loan_amount + (loan_amount * interest_rate / 1000000);

        return total_amount_to_pay;
    }

    pub fn get_loan_token(env: Env) -> Address {
        let loan_token = env.storage().instance().get(&DataKey::LoanToken).unwrap();

        loan_token
    }

    pub fn get_collateral_token(env: Env) -> Address {
        let collateral_token = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();

        collateral_token
    }

    pub fn get_total_supply(env: Env) -> i128 {
        let total_supply: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        total_supply
    }

    pub fn get_borrowers(env: Env) -> Vec<Borrower> {
        let borrowers: Vec<Borrower> = env
            .storage()
            .instance()
            .get(&DataKey::Borrowers)
            .unwrap_or(Vec::new(&env));

        borrowers
    }
}

#[cfg(test)]
mod test;
