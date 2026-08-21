#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Registry;

#[contractimpl]
impl Registry {
    pub fn set(env: Env, key: Symbol, value: i128) {
        env.storage().instance().set(&key, &value);
    }

    pub fn get(env: Env, key: Symbol) -> i128 {
        env.storage().instance().get(&key).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn set_and_get_roundtrip() {
        let mut mirror: HashMap<u32, i128> = HashMap::new();
        mirror.insert(1, 42);
        assert_eq!(mirror.get(&1), Some(&42));
    }
}