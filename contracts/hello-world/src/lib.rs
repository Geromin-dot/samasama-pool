#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Shift(Address), // Maps a specific driver's address to their active transit shift log
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShiftConfig {
    pub operator: Address,
    pub token: Address,
    pub target_boundary: i128,
    pub current_collected: i128,
    pub is_settled: bool,
}

#[contract]
pub struct PasadaLedgerContract;

#[contractimpl]
impl PasadaLedgerContract {
    /// Opens a new active transit shift, tying a driver to an operator with a set target boundary fee.
    pub fn start_shift(env: Env, driver: Address, operator: Address, token: Address, target_boundary: i128) {
        driver.require_auth();
        
        let key = DataKey::Shift(driver.clone());
        if env.storage().persistent().has(&key) {
            let existing: ShiftConfig = env.storage().persistent().get(&key).unwrap();
            if !existing.is_settled {
                panic!("Driver already has an active, unsettled transit shift.");
            }
        }

        let config = ShiftConfig {
            operator,
            token,
            target_boundary,
            current_collected: 0,
            is_settled: false,
        };
        
        env.storage().persistent().set(&key, &config);
    }

    /// Processes micro-payments from passengers or fare aggregators directly into the shift contract pool.
    pub fn deposit_fare(env: Env, driver: Address, passenger: Address, amount: i128) {
        passenger.require_auth();
        
        let key = DataKey::Shift(driver.clone());
        if !env.storage().persistent().has(&key) {
            panic!("No active transit shift found for this driver.");
        }

        let mut config: ShiftConfig = env.storage().persistent().get(&key).unwrap();
        if config.is_settled {
            panic!("Cannot deposit fares to an already settled transit shift.");
        }

        // Pull stablecoin fare from passenger account into contract custody
        let client = token::Client::new(&env, &config.token);
        client.transfer(&passenger, &env.current_contract_address(), &amount);

        config.current_collected += amount;
        env.storage().persistent().set(&key, &config);
    }

    /// Programmatically processes the boundary breakdown: splitting target dues to operator and surplus to driver.
    pub fn settle_shift(env: Env, driver: Address) {
        driver.require_auth();
        
        let key = DataKey::Shift(driver.clone());
        if !env.storage().persistent().has(&key) {
            panic!("No active shift registration found for this driver.");
        }

        let mut config: ShiftConfig = env.storage().persistent().get(&key).unwrap();
        if config.is_settled {
            panic!("This shift configuration has already been fully processed.");
        }

        config.is_settled = true;
        env.storage().persistent().set(&key, &config);

        let token_client = token::Client::new(&env, &config.token);
        
        if config.current_collected >= config.target_boundary {
            // Operator receives full boundary dues
            token_client.transfer(&env.current_contract_address(), &config.operator, &config.target_boundary);
            
            // Driver pockets the remaining surplus earnings instantly
            let surplus = config.current_collected - config.target_boundary;
            if surplus > 0 {
                token_client.transfer(&env.current_contract_address(), &driver, &surplus);
            }
        } else {
            // Under-collection path: Operator receives whatever total amount was managed during the route
            if config.current_collected > 0 {
                token_client.transfer(&env.current_contract_address(), &config.operator, &config.current_collected);
            }
        }
    }

    /// Read function to view precise live tracking configurations of a driver's daily shift log.
    pub fn get_shift(env: Env, driver: Address) -> ShiftConfig {
        let key = DataKey::Shift(driver);
        if !env.storage().persistent().has(&key) {
            panic!("No registered shift records found.");
        }
        env.storage().persistent().get(&key).unwrap()
    }
}