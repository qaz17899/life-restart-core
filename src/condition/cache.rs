//! Condition parsing cache - Optimized with faster hashing

use crate::condition::ast::AstNode;
use crate::condition::parser;
use crate::error::Result;
use ahash::AHashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

/// Global condition cache with fast hashing (ahash)
static CONDITION_CACHE: Lazy<RwLock<AHashMap<String, AstNode>>> = Lazy::new(|| {
    let map = AHashMap::with_capacity(2048);
    RwLock::new(map)
});

/// Get or parse a condition string, using cache for repeated conditions
#[inline]
pub fn get_or_parse(condition: &str) -> Result<AstNode> {
    // Fast path: check read lock first
    {
        let cache = CONDITION_CACHE.read();
        if let Some(ast) = cache.get(condition) {
            return Ok(ast.clone());
        }
    }

    // Slow path: parse and cache
    let ast = parser::parse(condition)?;

    {
        let mut cache = CONDITION_CACHE.write();
        cache.insert(condition.to_string(), ast.clone());
    }

    Ok(ast)
}

/// Check a condition against a PropertyState, using cached AST
#[inline]
pub fn check_condition(condition: &str, state: &crate::property::PropertyState) -> Result<bool> {
    if condition.is_empty() {
        return Ok(true);
    }

    let ast = get_or_parse(condition)?;
    Ok(crate::condition::evaluator::check(&ast, state))
}

/// Clear the condition cache (useful for testing)
#[allow(dead_code)]
pub fn clear_cache() {
    let mut cache = CONDITION_CACHE.write();
    cache.clear();
}

/// Get cache statistics
#[allow(dead_code)]
pub fn cache_size() -> usize {
    let cache = CONDITION_CACHE.read();
    cache.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::PropertyState;

    #[test]
    fn test_cache_hit() {
        clear_cache();

        let state = PropertyState {
            chr: 10,
            ..Default::default()
        };

        // First call - cache miss
        let result1 = check_condition("CHR>5", &state).unwrap();
        assert!(result1);
        assert_eq!(cache_size(), 1);

        // Second call - cache hit
        let result2 = check_condition("CHR>5", &state).unwrap();
        assert!(result2);
        assert_eq!(cache_size(), 1);
    }

    #[test]
    fn test_empty_condition() {
        let state = PropertyState::default();
        let result = check_condition("", &state).unwrap();
        assert!(result);
    }
}
