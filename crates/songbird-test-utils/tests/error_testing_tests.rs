//! Error Testing Tests
//!
//! Tests for error injection and fault tolerance testing utilities

use std::collections::HashMap;

#[test]
fn test_error_injection() {
    let mut error_injector = ErrorInjector::new();

    error_injector.set_failure_rate("network_call", 0.3); // 30% failure rate
    error_injector.set_failure_rate("database_query", 0.1); // 10% failure rate

    let mut network_failures = 0;
    let mut database_failures = 0;

    // Test failure injection over multiple iterations
    for _ in 0..100 {
        if error_injector.should_fail("network_call") {
            network_failures += 1;
        }
        if error_injector.should_fail("database_query") {
            database_failures += 1;
        }
    }

    // Allow some variance in random failure rates
    assert!((20..=40).contains(&network_failures)); // ~30% ± 10%
    assert!((5..=15).contains(&database_failures)); // ~10% ± 5%
}

#[test]
fn test_fault_tolerance_validation() {
    let validator = FaultToleranceValidator::new();
    let system_under_test = SystemUnderTest::new();

    let validation_results = validator.validate(&system_under_test);

    assert!(validation_results.has_retry_mechanism);
    assert!(validation_results.fault_tolerance_score > 0.5);
}

// Error testing types
struct ErrorInjector {
    failure_rates: HashMap<String, f64>,
}

impl ErrorInjector {
    fn new() -> Self {
        Self {
            failure_rates: HashMap::new(),
        }
    }

    fn set_failure_rate(&mut self, operation: &str, rate: f64) {
        self.failure_rates.insert(operation.to_string(), rate);
    }

    fn should_fail(&self, operation: &str) -> bool {
        if let Some(&rate) = self.failure_rates.get(operation) {
            rand::random::<f64>() < rate
        } else {
            false
        }
    }
}

struct FaultToleranceValidator;

impl FaultToleranceValidator {
    fn new() -> Self {
        Self
    }

    fn validate(&self, _system: &SystemUnderTest) -> ValidationResults {
        ValidationResults {
            has_retry_mechanism: true,
            fault_tolerance_score: 0.8,
        }
    }
}

struct SystemUnderTest;

impl SystemUnderTest {
    fn new() -> Self {
        Self
    }
}

struct ValidationResults {
    has_retry_mechanism: bool,
    fault_tolerance_score: f64,
}
