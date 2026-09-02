// Test 1: Simple #[cfg(test)] should work
#[cfg(test)]
mod tests {
    use std::collections::HashMap; // Should be ignored
    
    #[test]
    fn test_hashmap() {
        let map: HashMap<i32, i32> = HashMap::new();
        assert_eq!(map.len(), 0);
    }
}

// Test 2: Outside cfg(test) should trigger error
fn bad_function() {
    use std::collections::HashMap; // Should be caught
    let _map: HashMap<i32, i32> = HashMap::new();
}
