#![cfg(test)]

use std::collections::HashMap;

#[test]
fn builds_a_lookup_table() {
    let mut table: HashMap<u32, i128> = HashMap::new();
    table.insert(1, 42);
    println!("table has {} entries", table.len());
    assert_eq!(table.get(&1), Some(&42));
}