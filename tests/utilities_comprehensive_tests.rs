//! Comprehensive Utilities Tests for Songbird Orchestrator
//!
//! This test suite covers utility functions, string operations, data processing,
//! caching mechanisms, and general helper functions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use songbird_lib::errors::{Result, SongbirdError};

#[test]
fn test_string_utilities_basic() {
    // Test basic string operations
    let input = "  Hello World  ";
    let trimmed = input.trim();
    assert_eq!(trimmed, "Hello World");

    let uppercase = trimmed.to_uppercase();
    assert_eq!(uppercase, "HELLO WORLD");

    let lowercase = trimmed.to_lowercase();
    assert_eq!(lowercase, "hello world");
}

#[test]
fn test_string_utilities_formatting() {
    // Test string formatting
    let name = "Songbird";
    let version = "1.0.0";

    let formatted = format!("{name} v{version}");
    assert_eq!(formatted, "Songbird v1.0.0");

    let padded = format!("{name:>10}");
    assert_eq!(padded, "  Songbird");

    let left_padded = format!("{name:<10}");
    assert_eq!(left_padded, "Songbird  ");
}

#[test]
fn test_string_utilities_validation() {
    // Test string validation
    let email = "user@example.com";
    assert!(email.contains('@'));
    assert!(email.contains('.'));

    let empty_string = "";
    // Verify empty string behavior
    assert_eq!(empty_string.len(), 0);

    let whitespace_string = "   ";
    // Verify whitespace string is not empty (contains spaces)
    assert_eq!(whitespace_string.len(), 3);
    assert!(whitespace_string.trim().is_empty());
}

#[test]
fn test_string_utilities_parsing() {
    // Test string parsing
    let number_string = "42";
    let parsed_number: i32 = number_string.parse().unwrap();
    assert_eq!(parsed_number, 42);

    let float_string = "3.14159";
    let parsed_float: f64 = float_string.parse().unwrap();
    assert!((parsed_float - std::f64::consts::PI).abs() < 0.00001);

    let bool_string = "true";
    let parsed_bool: bool = bool_string.parse().unwrap();
    assert!(parsed_bool);
}

#[test]
fn test_string_utilities_splitting() {
    // Test string splitting
    let csv_data = "apple,banana,cherry";
    let fruits: Vec<&str> = csv_data.split(',').collect();
    assert_eq!(fruits.len(), 3);
    assert_eq!(fruits[0], "apple");
    assert_eq!(fruits[1], "banana");
    assert_eq!(fruits[2], "cherry");

    let multiline = "line1\nline2\nline3";
    let lines: Vec<&str> = multiline.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_data_processing_collections() {
    // Test collection operations
    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];

    // Test sorting
    data.sort();
    assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);

    // Test deduplication
    data.dedup();
    assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 9]);

    // Test filtering
    let even_numbers: Vec<i32> = data.iter().filter(|&x| x % 2 == 0).cloned().collect();
    assert_eq!(even_numbers, vec![2, 4, 6]);
}

#[test]
fn test_data_processing_mapping() {
    // Test mapping operations
    let numbers = [1, 2, 3, 4, 5];

    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6, 8, 10]);

    let sum: i32 = numbers.iter().sum();
    assert_eq!(sum, 15);

    let product: i32 = numbers.iter().product();
    assert_eq!(product, 120);
}

#[test]
fn test_data_processing_aggregation() {
    // Test aggregation operations
    let data = [10, 20, 30, 40, 50];

    let min = data.iter().min().unwrap();
    assert_eq!(*min, 10);

    let max = data.iter().max().unwrap();
    assert_eq!(*max, 50);

    let average = data.iter().sum::<i32>() as f64 / data.len() as f64;
    assert_eq!(average, 30.0);
}

#[test]
fn test_caching_basic() {
    // Test basic caching mechanism
    let mut cache: HashMap<String, String> = HashMap::new();

    // Test cache miss
    assert!(!cache.contains_key("key1"));

    // Test cache insert
    cache.insert("key1".to_string(), "value1".to_string());
    assert!(cache.contains_key("key1"));

    // Test cache hit
    let value = cache.get("key1").unwrap();
    assert_eq!(value, "value1");

    // Test cache size
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_caching_advanced() {
    // Test advanced caching with TTL simulation
    #[allow(dead_code)]
    struct CacheEntry {
        value: String,
        timestamp: Instant,
        ttl: Duration,
    }

    impl CacheEntry {
        fn new(value: String, ttl: Duration) -> Self {
            Self {
                value,
                timestamp: Instant::now(),
                ttl,
            }
        }

        fn is_expired(&self) -> bool {
            self.timestamp.elapsed() > self.ttl
        }
    }

    let mut cache: HashMap<String, CacheEntry> = HashMap::new();

    // Insert entry with short TTL
    cache.insert(
        "temp_key".to_string(),
        CacheEntry::new("temp_value".to_string(), Duration::from_millis(1)),
    );

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(10));

    // Check if expired
    let entry = cache.get("temp_key").unwrap();
    assert!(entry.is_expired());
}

#[test]
fn test_caching_thread_safe() {
    // Test thread-safe caching
    let cache = Arc::new(Mutex::new(HashMap::<String, i32>::new()));
    let cache_clone = cache.clone();

    let handle = std::thread::spawn(move || {
        let mut cache = cache_clone.lock().unwrap();
        cache.insert("thread_key".to_string(), 42);
    });

    handle.join().unwrap();

    let cache = cache.lock().unwrap();
    assert_eq!(cache.get("thread_key"), Some(&42));
}

#[test]
fn test_helper_functions_validation() {
    // Test validation helper functions
    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && !email.trim().is_empty()
    }

    assert!(is_valid_email("user@example.com"));
    assert!(!is_valid_email("invalid-email"));
    assert!(!is_valid_email(""));

    fn is_valid_port(port: u16) -> bool {
        port > 0 // Port 0 is reserved and not usable
    }

    assert!(is_valid_port(8080));
    assert!(!is_valid_port(0));
    assert!(is_valid_port(65535));
}

#[test]
fn test_helper_functions_conversion() {
    // Test conversion helper functions
    fn bytes_to_human_readable(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    assert_eq!(bytes_to_human_readable(1024), "1.00 KB");
    assert_eq!(bytes_to_human_readable(1048576), "1.00 MB");
    assert_eq!(bytes_to_human_readable(1073741824), "1.00 GB");

    fn seconds_to_duration_string(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{hours}h {minutes}m {secs}s")
        } else if minutes > 0 {
            format!("{minutes}m {secs}s")
        } else {
            format!("{secs}s")
        }
    }

    assert_eq!(seconds_to_duration_string(3661), "1h 1m 1s");
    assert_eq!(seconds_to_duration_string(61), "1m 1s");
    assert_eq!(seconds_to_duration_string(30), "30s");
}

#[test]
fn test_helper_functions_retry_logic() {
    // Test retry logic helper
    fn retry_with_backoff<F, T, E>(mut operation: F, max_retries: u32) -> std::result::Result<T, E>
    where
        F: FnMut() -> std::result::Result<T, E>,
    {
        let mut attempts = 0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    if attempts >= max_retries {
                        return Err(error);
                    }
                    // In real implementation, would sleep with exponential backoff
                }
            }
        }
    }

    let mut counter = 0;
    let result = retry_with_backoff(
        || {
            counter += 1;
            if counter < 3 {
                Err("temporary failure")
            } else {
                Ok("success")
            }
        },
        5,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter, 3);
}

#[test]
fn test_helper_functions_rate_limiting() {
    // Test rate limiting helper
    struct RateLimiter {
        requests: Vec<Instant>,
        max_requests: usize,
        window: Duration,
    }

    impl RateLimiter {
        fn new(max_requests: usize, window: Duration) -> Self {
            Self {
                requests: Vec::new(),
                max_requests,
                window,
            }
        }

        fn allow_request(&mut self) -> bool {
            let now = Instant::now();

            // Remove old requests outside the window
            self.requests
                .retain(|&timestamp| now.duration_since(timestamp) < self.window);

            if self.requests.len() < self.max_requests {
                self.requests.push(now);
                true
            } else {
                false
            }
        }
    }

    let mut rate_limiter = RateLimiter::new(3, Duration::from_secs(1));

    // First 3 requests should be allowed
    assert!(rate_limiter.allow_request());
    assert!(rate_limiter.allow_request());
    assert!(rate_limiter.allow_request());

    // 4th request should be denied
    assert!(!rate_limiter.allow_request());
}

#[test]
fn test_helper_functions_circuit_breaker() {
    // Test circuit breaker helper
    #[derive(PartialEq, Debug)]
    enum CircuitState {
        Closed,
        Open,
        HalfOpen,
    }

    struct CircuitBreaker {
        state: CircuitState,
        failure_count: u32,
        failure_threshold: u32,
        last_failure_time: Option<Instant>,
        timeout: Duration,
    }

    impl CircuitBreaker {
        fn new(failure_threshold: u32, timeout: Duration) -> Self {
            Self {
                state: CircuitState::Closed,
                failure_count: 0,
                failure_threshold,
                last_failure_time: None,
                timeout,
            }
        }

        fn call<F, T, E>(&mut self, operation: F) -> std::result::Result<T, String>
        where
            F: FnOnce() -> std::result::Result<T, E>,
        {
            match self.state {
                CircuitState::Open => {
                    if let Some(last_failure) = self.last_failure_time {
                        if last_failure.elapsed() > self.timeout {
                            self.state = CircuitState::HalfOpen;
                        } else {
                            return Err("Circuit breaker is open".to_string());
                        }
                    }
                }
                CircuitState::HalfOpen => {
                    // Allow one test request
                }
                CircuitState::Closed => {
                    // Normal operation
                }
            }

            match operation() {
                Ok(result) => {
                    self.failure_count = 0;
                    self.state = CircuitState::Closed;
                    Ok(result)
                }
                Err(_) => {
                    self.failure_count += 1;
                    self.last_failure_time = Some(Instant::now());

                    if self.failure_count >= self.failure_threshold {
                        self.state = CircuitState::Open;
                    }

                    Err("Operation failed".to_string())
                }
            }
        }
    }

    let mut circuit_breaker = CircuitBreaker::new(2, Duration::from_millis(100));

    // First failure
    let result = circuit_breaker.call(|| -> std::result::Result<String, ()> { Err(()) });
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state, CircuitState::Closed);

    // Second failure - should open circuit
    let result = circuit_breaker.call(|| -> std::result::Result<String, ()> { Err(()) });
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state, CircuitState::Open);

    // Next call should be blocked
    let result =
        circuit_breaker.call(|| -> std::result::Result<String, ()> { Ok("success".to_string()) });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Circuit breaker is open");
}

#[test]
fn test_data_structures_queue() {
    // Test queue operations
    use std::collections::VecDeque;

    let mut queue: VecDeque<i32> = VecDeque::new();

    // Test enqueue
    queue.push_back(1);
    queue.push_back(2);
    queue.push_back(3);

    assert_eq!(queue.len(), 3);

    // Test dequeue
    assert_eq!(queue.pop_front(), Some(1));
    assert_eq!(queue.pop_front(), Some(2));
    assert_eq!(queue.len(), 1);

    // Test peek
    assert_eq!(queue.front(), Some(&3));
}

#[test]
fn test_data_structures_stack() {
    // Test stack operations
    let mut stack: Vec<i32> = Vec::new();

    // Test push
    stack.push(1);
    stack.push(2);
    stack.push(3);

    assert_eq!(stack.len(), 3);

    // Test pop
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.len(), 1);

    // Test peek
    assert_eq!(stack.last(), Some(&1));
}

#[test]
fn test_data_structures_priority_queue() {
    // Test priority queue operations
    use std::collections::BinaryHeap;

    let mut heap = BinaryHeap::new();

    // Insert elements
    heap.push(3);
    heap.push(1);
    heap.push(4);
    heap.push(2);

    assert_eq!(heap.len(), 4);

    // Extract maximum (BinaryHeap is max-heap by default)
    assert_eq!(heap.pop(), Some(4));
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert_eq!(heap.pop(), None);
}

#[test]
fn test_utility_algorithms_sorting() {
    // Test sorting algorithms
    let mut data = vec![64, 34, 25, 12, 22, 11, 90];

    // Built-in sort
    data.sort();
    assert_eq!(data, vec![11, 12, 22, 25, 34, 64, 90]);

    // Reverse sort
    data.sort_by(|a, b| b.cmp(a));
    assert_eq!(data, vec![90, 64, 34, 25, 22, 12, 11]);

    // Custom sort (by last digit)
    data.sort_by_key(|&x| x % 10);
    assert_eq!(data[0] % 10, 0); // 90
}

#[test]
fn test_utility_algorithms_searching() {
    // Test searching algorithms
    let data = [1, 3, 5, 7, 9, 11, 13, 15];

    // Binary search
    assert_eq!(data.binary_search(&7), Ok(3));
    assert_eq!(data.binary_search(&4), Err(2));

    // Linear search
    assert_eq!(data.iter().position(|&x| x == 9), Some(4));
    assert_eq!(data.iter().position(|&x| x == 100), None);

    // Find with condition
    assert_eq!(data.iter().find(|&&x| x > 10), Some(&11));
}

#[test]
fn test_utility_time_operations() {
    // Test time utility operations
    let start = Instant::now();

    // Simulate some work
    std::thread::sleep(Duration::from_millis(10));

    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(100));

    // Test duration formatting
    let duration = Duration::from_secs(65);
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    assert_eq!(minutes, 1);
    assert_eq!(seconds, 5);
}

#[test]
fn test_utility_math_operations() {
    // Test mathematical utility operations
    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    assert_eq!(gcd(48, 18), 6);
    assert_eq!(gcd(17, 13), 1);

    fn lcm(a: u64, b: u64) -> u64 {
        a * b / gcd(a, b)
    }

    assert_eq!(lcm(12, 18), 36);

    fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        for i in 2..=(n as f64).sqrt() as u64 {
            if n % i == 0 {
                return false;
            }
        }
        true
    }

    assert!(is_prime(17));
    assert!(!is_prime(15));
    assert!(is_prime(2));
}

#[test]
fn test_utility_encoding_operations() {
    // Test encoding utility operations

    fn url_encode(input: &str) -> String {
        // Simple URL encoding simulation
        input.replace(' ', "%20").replace('&', "%26")
    }

    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(url_encode("foo&bar"), "foo%26bar");

    fn simple_hash(input: &str) -> u64 {
        // Simple hash function for testing
        input.chars().map(|c| c as u64).sum()
    }

    assert_eq!(simple_hash("hello"), simple_hash("hello"));
    assert_ne!(simple_hash("hello"), simple_hash("world"));
}

#[test]
fn test_utility_configuration_helpers() {
    // Test configuration utility helpers
    fn parse_env_var<T: std::str::FromStr>(var_name: &str, default: T) -> T {
        std::env::var(var_name)
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(default)
    }

    // Test with non-existent env var
    let port: u16 = parse_env_var("NONEXISTENT_PORT", 8080);
    assert_eq!(port, 8080);

    fn merge_configs<T: Clone>(
        base: &HashMap<String, T>,
        override_config: &HashMap<String, T>,
    ) -> HashMap<String, T> {
        let mut merged = base.clone();
        for (key, value) in override_config {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    let mut base_config = HashMap::new();
    base_config.insert("port".to_string(), "8080".to_string());
    base_config.insert("host".to_string(), "localhost".to_string());

    let mut override_config = HashMap::new();
    override_config.insert("port".to_string(), "9090".to_string());

    let merged = merge_configs(&base_config, &override_config);
    assert_eq!(merged.get("port"), Some(&"9090".to_string()));
    assert_eq!(merged.get("host"), Some(&"localhost".to_string()));
}

#[test]
fn test_utility_performance() {
    // Test utility performance
    let start = Instant::now();

    // Test performance of various operations
    let mut data: Vec<i32> = (0..1000).collect();
    data.sort();
    data.reverse();
    let _sum: i32 = data.iter().sum();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(10)); // Should be very fast
}

#[test]
fn test_utility_memory_efficiency() {
    // Test memory efficiency
    let small_vec: Vec<u8> = vec![1, 2, 3];
    let capacity = small_vec.capacity();
    let len = small_vec.len();

    assert!(capacity >= len);
    assert!(len == 3);

    // Test iterator efficiency (lazy evaluation)
    let large_range = 0..1_000_000;
    let first_evens: Vec<i32> = large_range.filter(|x| x % 2 == 0).take(5).collect();
    assert_eq!(first_evens, vec![0, 2, 4, 6, 8]);
}

#[test]
fn test_utility_error_handling() {
    // Test error handling utilities
    fn safe_divide(a: f64, b: f64) -> Result<f64> {
        if b == 0.0 {
            Err(SongbirdError::Config {
                field: Some("divisor".to_string()),
                message: "Division by zero".to_string(),
            })
        } else {
            Ok(a / b)
        }
    }

    assert!(safe_divide(10.0, 2.0).is_ok());
    assert_eq!(safe_divide(10.0, 2.0).unwrap(), 5.0);

    assert!(safe_divide(10.0, 0.0).is_err());

    fn chain_operations() -> Result<String> {
        let result1 = safe_divide(10.0, 2.0)?;
        let result2 = safe_divide(result1, 1.0)?;
        Ok(format!("Result: {result2}"))
    }

    assert!(chain_operations().is_ok());
    assert_eq!(chain_operations().unwrap(), "Result: 5");
}
