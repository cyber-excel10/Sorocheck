use std::collections::HashMap as Map;

fn build_lookup() -> Map<u32, i128> {
    let mut map = Map::new();
    map.insert(1, 100);
    map
}