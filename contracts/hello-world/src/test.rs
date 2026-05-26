#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup_test_environment(env: &Env) -> (Address, Address, Address, Address, token::Client, token::StellarAssetClient) {
    let manager = Address::generate(env);
    let member_one = Address::generate(env);
    let recipient = Address::generate(env);
    
    // Deploy standard mock token representing PHPC stablecoin options
    let token_id = env.register_stellar_asset_contract(manager.clone());
    let token_client = token::Client::new(env, &token_id);
    let token_admin = token::StellarAssetClient::new(env, &token_id);
    
    // Fund participating pool members with standard stable balances
    token_admin.mint(&member_one, &10_000);
    token_admin.mint(&recipient, &10_000);
    
    (manager, member_one, recipient, token_id, token_client, token_admin)
}

#[test]
fn test_happy_path_pool_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (manager, member_one, recipient, token_id, token_client, _) = setup_test_environment(&env);
    let contract_id = env.register_contract(None, SamaSamaPoolContract);
    let client = SamaSamaPoolContractClient::new(&env, &contract_id);

    // Act: Initialize round with requirements for 2 members paying 2,000 each
    client.setup_pool(&manager, &token_id, &2_000, &2, &recipient);
    
    // Act: Process structured financial deposits from both members
    client.deposit_contribution(&member_one);
    client.deposit_contribution(&recipient);
    
    assert_eq!(token_client.balance(&contract_id), 4_000);

    // Act: Disburse the round funds to the scheduled beneficiary
    client.claim_pool(&recipient);
    
    // Assert: Balance verification checks
    assert_eq!(token_client.balance(&recipient), 12_000); // 10k baseline + 4k pool - 2k deposit
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "An active savings pool cycle is already registered.")]
fn test_edge_case_duplicate_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (manager, _, recipient, token_id, _, _) = setup_test_environment(&env);
    let contract_id = env.register_contract(None, SamaSamaPoolContract);
    let client = SamaSamaPoolContractClient::new(&env, &contract_id);

    client.setup_pool(&manager, &token_id, &2_000, &3, &recipient);
    // Malicious or errant configuration overlap injection attempt
    client.setup_pool(&manager, &token_id, &1_000, &2, &recipient);
}

#[test]
fn test_state_verification_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (manager, _, recipient, token_id, _, _) = setup_test_environment(&env);
    let contract_id = env.register_contract(None, SamaSamaPoolContract);
    let client = SamaSamaPoolContractClient::new(&env, &contract_id);

    client.setup_pool(&manager, &token_id, &5_000, &4, &recipient);
    
    let config = client.get_pool();
    assert_eq!(config.manager, manager);
    assert_eq!(config.amount_per_member, 5_000);
    assert_eq!(config.required_deposits, 4);
    assert_eq!(config.current_deposits, 0);
    assert_eq!(config.is_claimed, false);
}

#[test]
#[should_panic(expected = "Cannot claim payout; pool contributions are still incomplete.")]
fn test_edge_case_premature_claim_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (manager, member_one, recipient, token_id, _, _) = setup_test_environment(&env);
    let contract_id = env.register_contract(None, SamaSamaPoolContract);
    let client = SamaSamaPoolContractClient::new(&env, &contract_id);

    client.setup_pool(&manager, &token_id, &2_000, &2, &recipient);
    client.deposit_contribution(&member_one);
    
    // Attempting extraction prior to checking out missing peer requirements
    client.claim_pool(&recipient);
}

#[test]
#[should_panic(expected = "The pooled payout for this cycle has already been claimed.")]
fn test_edge_case_cannot_double_claim() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (manager, member_one, recipient, token_id, _, _) = setup_test_environment(&env);
    let contract_id = env.register_contract(None, SamaSamaPoolContract);
    let client = SamaSamaPoolContractClient::new(&env, &contract_id);

    client.setup_pool(&manager, &token_id, &2_000, &2, &recipient);
    client.deposit_contribution(&member_one);
    client.deposit_contribution(&recipient);
    
    client.claim_pool(&recipient);
    // Malicious secondary execution loop target against exhausted state allocations
    client.claim_pool(&recipient);
}