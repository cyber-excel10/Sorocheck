#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn increment(env: Env, key: Symbol, amount: i128) -> i128 {
        let current: i128 = env.storage().instance().get(&key).unwrap_or(0);
        let updated = current + amount;
        env.storage().instance().set(&key, &updated);
        updated
    }

    pub fn get(env: Env, key: Symbol) -> i128 {
        env.storage().instance().get(&key).unwrap_or(0)
    }
}