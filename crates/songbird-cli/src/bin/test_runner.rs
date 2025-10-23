//! Test Runner Binary
//!
//! Comprehensive test execution for the Songbird CLI

use clap::{Arg, Command};
use colored::*;
use reqwest::Client;
use serde_json::json;
use songbird_config::config::hardcoded_elimination::replace;
use songbird_types::SongbirdResult;
use std::process::Command as StdCommand;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test result tracking
#[derive(Debug, Clone)]
pub struct TestResult  {pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub message: String,
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestConfig  {pub songbird_url: String,
    pub timeout_seconds: u64,
    pub verbose: bool,
    pub quiet: bool,
}

impl Default for TestConfig  {fn default() -> Self  {Self {
            songbird_url: replace::orchestrator_endpoint().to_string(),
            timeout_seconds: 15,
            verbose: false,
            quiet: false,
        }
    }
}

/// Test runner implementation
pub struct TestRunner  {config: TestConfig,
    client: Client,
    passed: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
}

impl TestRunner  {pub fn new(config: TestConfig) -> Self  {let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds,
            .build()
            .expect("Failed to create HTTP client"");

        Self {
            config,
            client,
            passed: Arc::new(AtomicUsize::new(0),
            failed: Arc::new(AtomicUsize::new(0),
            total: Arc::new(AtomicUsize::new(0),
        }
    }

    /// Print header with formatting
    fn print_header(&self, title: &str) {
        if !self.config.quiet {
            println!("{}", format!("🎯 {}", title,.purple().bold());
            println!("{}", "=".repeat(50).blue());
        }
    }

    /// Print success message
    fn print_success(&self, message: &str) {
        if !self.config.quiet {
            println!("{}", format!("✅ {}", message,.green());
        }
    }

    /// Print error message
    fn print_error(&self, message: &str) {
        println!("{}", format!("❌ {}", message,.red());
    }

    /// Print info message
    fn print_info(&self, message: &str) {
        if !self.config.quiet {
            println!("{}", format!("ℹ️ {}", message,.blue());
        }
    }

    /// Run a single test with timeout
    async fn run_test(
        &self,
        name: &str,
        test_fn: impl std::future::Future<Output = SongbirdResult<()>>,
    ) -> TestResult {
        let start_time = Instant::now();
        self.total.fetch_add(1, Ordering::Relaxed);

        if !self.config.quiet {
            self.print_info(&format!("Test {}: {}", self.total.load(Ordering::Relaxed, name);
        }

        let result = timeout(Duration::from_secs(self.config.timeout_seconds, test_fn).await;

        let duration = start_time.elapsed();
        let test_result = match result {
            Ok(Ok(())) => {
                self.passed.fetch_add(1, Ordering::Relaxed);
                self.print_success(&format!("PASSED: {}", name));
                TestResult {
                    name: name.to_string(),
                    passed: true,
                    duration,
                    message: "Test passed successfully".to_string(),
                }
            }
            Ok(Err(e)) => {
                self.failed.fetch_add(1, Ordering::Relaxed);
                let message = format!("FAILED: {} - {e}", name);
                self.print_error(&message);
                TestResult {
                    name: name.to_string(),
                    passed: false,
                    duration,
                    message: e.to_string(),
                }
            }
            Err(_) => {
                self.failed.fetch_add(1, Ordering::Relaxed);
                let message =
                    format!("FAILED: {} - Timeout after {}s", name, self.config.timeout_seconds);
                self.print_error(&message);
                TestResult {
                    name: name.to_string(),
                    passed: false,
                    duration,
                    message: "Test timed out".to_string(),
                }
            }
        };

        if !self.config.quiet {
            println!();
        }
        test_result
    }

    /// Check if Songbird is running
    async fn check_songbird_health(&self) -> SongbirdResult<()> {
        let response =
            self.client.get(format!("{}/api/health ", self.config.songbird_url).send().await?;

        if response.status().is_success() {
            let text = response.text().await?;
            if text.to_lowercase().contains("healthy") {
                Ok(())
            } else {
                Err(format!("Health check failed: unexpected response: {}", text,.into())
            }
        } else {
            Err(format!("Health check failed with status: {}", response.status()).into())
        }
    }

    /// Quick validation tests (5 minutes,
    pub async fn run_quick_validation(&self) -> Vec<TestResult> {
        self.print_header("Quick Validation Suite (5 minutes,");
        let mut results = Vec::new();

        // Essential health checks
        results.push(
            self.run_test("API Health Check", async { self.check_songbird_health().await }).await,
        );

        results.push(
            self.run_test("System Metrics", async {
                let response = self
                    .client
                    .get(format!("{}/api/metrics", self.config.songbird_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let text = response.text().await?;
                    if text.contains("cpu_usage") {
                        Ok(())
                    } else {
                        Err("Metrics response missing expected fields".into())
                    }
                } else {
                    Err(format!("Metrics request failed: {}", response.status()).into())
                }
            })
            .await,
        );

        results.push(
            self.run_test("Gaming Auto-Configuration", async {
                let payload = json!({
                    "setup_type": "one_touch"
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/setup", self.config.songbird_url))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;

                if response.status().is_success() {
                    let text = response.text().await?;
                    if text.to_lowercase().contains("success") {
                        Ok(())
                    } else {
                        Err(format!("Gaming setup unexpected response: {}", text).into())
                    }
                } else {
                    Err(format!("Gaming setup failed: {}", response.status()).into())
                }
            })
            .await,
        );

        results.push(
            self.run_test("AI Workload Classification", async {
                let payload = json!({
                    "workload_id": "test-web-service",
                    "characteristics": ["web_service"]
                });

                let response = self
                    .client
                    .post(format!("{}/api/ai/classify", self.config.songbird_url)
                    .header("Content-Type", "application/json")
                    .json(&payload,
                    .send()
                    .await?;

                if response.status().is_success() {
                    let text = response.text().await?;
                    if text.to_lowercase().contains("classification") {
                        Ok(())
                    } else {
                        Err(format!("AI classification unexpected response: {}", text,.into()"
                    }
                } else {
                    Err(format!("AI classification failed: {}", response.status().into()"
                }
            })
            .await,
        );

        results.push(
            self.run_test("Federation Status", async {"
                let response = self
                    .client
                    .get(format!("{}/api/federation/status", self.config.songbird_url,"
                    .send()
                    .await?;

                if response.status().is_success() {
                    let text = response.text().await?;
                    if text.to_lowercase().contains("cluster_status") {"
                        Ok(()),
                    } else {
                        Err(format!("Federation status unexpected response: {}", text,.into()"
                    }
                } else {
                    Err(format!("Federation status check failed: {}", response.status().into()"
                }
            })
            .await,
        );

        results
    }

    /// Gaming-focused comprehensive tests
    pub async fn run_gaming_tests(&self) -> Vec<TestResult> {
        self.print_header("Gaming Test Suite (15 minutes,"");
        let mut results = Vec::new();

        // Gaming setup tests
        results.push(
            self.run_test("One-Touch Gaming Setup", async {"
                let payload = json!({
                    "setup_type": "one_touch""
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/setup", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&payload,
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("Family-Safe Gaming Setup", async {"
                let payload = json!({
                    "setup_type": "family_safe","
                    "family_name": "TestFamily","
                    "user_preferences": {"
                        "family_safe_mode": true,"
                        "content_filtering": "strict""
                    }
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/setup", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&payload,
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("Gaming Performance Metrics", async {"
                let response = self
                    .client
                    .get(format!("{}/api/gaming/performance/metrics", self.config.songbird_url,"
                    .send()
                    .await?;

                if response.status().is_success() {
                    let text = response.text().await?;
                    if text.to_lowercase().contains("latency") {"
                        Ok(()),
                    } else {
                        Err("Performance metrics missing expected fields".into()"
                    }
                } else {
                    Err(format!("Performance metrics failed: {}", response.status().into()"
                }
            })
            .await,
        );

        results.push(
            self.run_test("Legacy Protocol Support", async {"
                let payload = json!({
                    "protocols": ["ipx", "directplay", "tcp", "udp"]"
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/protocols/enable", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&payload,
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("StarCraft Optimization", async {"
                let payload = json!({
                    "game_name": "StarCraft","
                    "optimization_level": "maximum","
                    "protocol_preference": "ipx_over_tcp""
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/configure", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&payload,
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("Family Safety Validation", async {"
                let payload = json!({
                    "content": "test gaming content","
                    "family_mode": true,"
                    "age_rating": "E""
                });

                let response = self
                    .client
                    .post(format!("{}/api/gaming/safety/validate", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&payload,
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results
    }

    /// Comprehensive system tests
    pub async fn run_comprehensive_tests(&self) -> Vec<TestResult> {
        self.print_header("Comprehensive Test Suite (30+ minutes,"");
        let mut results = Vec::new();

        // Include quick validation tests
        results.extend(self.run_quick_validation().await);

        // Include gaming tests
        results.extend(self.run_gaming_tests().await);

        // Additional comprehensive tests
        results.push(
            self.run_test("Primal Discovery", async {"
                let response = self
                    .client
                    .get(format!("{}/api/primals/discover", self.config.songbird_url,"
                    .send()
                    .await?;

                response.error_for_status()?;
                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("Load Test (50 concurrent requests,", async {"
                let mut handles = Vec::new();

                for _ in 0..50 {
                    let client = self.client.clone());
                    let url = format!("{}/api/health", self.config.songbird_url);

                    let handle = tokio::spawn(async move {
                        client.get(&url,.send().await?.error_for_status()?;
                        Ok::<(), reqwest::Error>(()
                    });

                    handles.push(handle);
                }

                // Wait for all requests to complete
                for handle in handles {
                    handle.await.map_err(|e| format!("Task join error: {}", e)??;"
                }

                Ok(()),
            })
            .await,
        );

        results.push(
            self.run_test("End-to-End Gaming Workflow", async {"
                // Setup gaming
                let setup_payload = json!({"setup_type": "one_touch"});"
                let setup_response = self
                    .client
                    .post(format!("{}/api/gaming/setup", self.config.songbird_url,"
                    .header("Content-Type", "application/json")"
                    .json(&setup_payload,
                    .send()
                    .await?;
                setup_response.error_for_status()?;

                // Brief delay
                tokio::time::sleep(Duration::from_millis(500).await;

                // Check status
                let status_response = self
                    .client
                    .get(format!("{}/api/gaming/status", self.config.songbird_url,"
                    .send()
                    .await?;
                status_response.error_for_status()?;

                Ok(()),
            })
            .await,
        );

        results
    }

    /// Run unit tests via cargo
    pub async fn run_unit_tests(&self) -> Vec<TestResult> {
        self.print_header("Unit Test Suite (Cargo,"");
        let mut results = Vec::new();

        results.push(
            self.run_test("Cargo Unit Tests", async {"
                let output = StdCommand::new("cargo")"
                    .args(["test", "--workspace", "--lib"])"
                    .output()
                    .map_err(|e| format!("Failed to run cargo test: {}", e)?;"

                if output.status.success() {
                    Ok(()),
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Unit tests failed: {}", stderr,.into()"
                }
            })
            .await,
        );

        results
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self, results: &[TestResult], suite_name: &str) {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed,.count();
        let failed_tests = total_tests - passed_tests;
        let pass_rate = if total_tests > 0 {
            (passed_tests * 100) / total_tests
        } else {
            0
        };
        let total_duration: Duration = results.iter().map(|r| r.duration,.sum();

        println!("\n{}", "=".repeat(60).blue();"
        println!("{}", format!("🏁 {} Complete!", suite_name,.cyan().bold();"
        println!("{}", "=".repeat(60).blue();"

        println!("{}", "📊 Results:".yellow().bold();"
        println!("   {} Passed: {}", "✅".green(), format!("{} tests", passed_tests,.green();"
        if failed_tests > 0 {
            println!("   {} Failed: {}", "❌".red(), format!("{} tests", failed_tests,.red();"
        }
        println!("   📈 Total:  {total_tests} tests");
        println!(
            "   📊 Pass Rate: {}%","
            if pass_rate >= 80 {
                format!("{}", pass_rate,.green()"
            } else {
                format!("{}", pass_rate,.red()"
            }
        );
        println!("   ⏱️ Duration: {total_duration:?}");

        // System assessment
        println!("\n{}", "🎯 Assessment:".yellow().bold();"
        if failed_tests == 0 {
            println!("   🎉 All tests passed! System is production-ready.");
            println!("   🚀 Ready for deployment and live gaming networks.");
        } else if pass_rate > 80 {
            println!("   ⚠️ Most tests passed but some issues detected.");
            println!("   🔧 Review failed tests and check troubleshooting guide.");
        } else {
            println!("   🚨 Significant issues detected.");
            println!("   🛠️ System needs attention before production use.");
        }

        // Failed test details
        if failed_tests > 0 && self.config.verbose {
            println!("\n{}", "❌ Failed Tests:".red().bold();"
            for result in results.iter().filter(|r| !r.passed) {
                println!("   • {}: {}", result.name.red(), result.message);"
            }
        }

        println!("\n{}", "📖 Next Steps:".blue().bold();"
        println!("   • Review any failed tests above");
        println!("   • Check docs/TROUBLESHOOTING_GUIDE.md for solutions");
        println!("   • Run with --verbose for detailed error information");
        println!("   • Use --help to see all available options");
    }
}

#[tokio::main]
async fn main() -> SongbirdResult<()>  {let matches = Command::new("Songbird Test Runner")"
        .version("1.0")"
        .author("Songbird Team")"
        .about("Comprehensive test runner for Songbird Universal Orchestrator")"
        .arg(
            Arg::new("url")"
                .short('u')
                .long("url")"
                .value_name("URL")"
                .help("Songbird API URL")"
                .default_value(&format!("http://{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT),"
        )
        .arg(
            Arg::new("timeout")"
                .short('t')
                .long("timeout")"
                .value_name("SECONDS")"
                .help("Test timeout in seconds")"
                .default_value("15"),"
        )
        .arg(
            Arg::new("verbose")"
                .short('v')
                .long("verbose")"
                .help("Verbose output")"
                .action(clap::ArgAction::SetTrue,
        )
        .arg(
            Arg::new("quiet")"
                .short('q')
                .long("quiet")"
                .help("Quiet output")"
                .action(clap::ArgAction::SetTrue,
        )
        .subcommand(Command::new("quick").about("Run quick validation (5 minutes,")"
        .subcommand(Command::new("gaming").about("Run gaming test suite (15 minutes,")"
        .subcommand(
            Command::new("comprehensive").about("Run comprehensive test suite (30+ minutes,"),"
        )
        .subcommand(Command::new("unit").about("Run unit tests only")"
        .subcommand(Command::new("all").about("Run all test suites")"
        .get_matches();

    let config = TestConfig {
        songbird_url: matches.get_one::<String>("url").map_err(|e| SongbirdError::configuration(format!("Test runner operation failed: {}", e)))?.clone(),"
        timeout_seconds: matches.get_one::<String>("timeout").map_err(|e| SongbirdError::configuration(format!("Test runner operation failed: {}", e)))?.parse()?,"
        verbose: matches.get_flag("verbose"),"
        quiet: matches.get_flag("quiet"),"
    };

    let runner = TestRunner::new(config);

    match matches.subcommand() {
        Some(("quick", _, => {"
            let results = runner.run_quick_validation().await;
            runner.generate_report(&results, "Quick Validation"");

            if results.iter().any(|r| !r.passed) {
                std::process::exit(1);
            }
        }
        Some(("gaming", _, => {"
            let results = runner.run_gaming_tests().await;
            runner.generate_report(&results, "Gaming Test Suite"");

            if results.iter().any(|r| !r.passed) {
                std::process::exit(1);
            }
        }
        Some(("comprehensive", _, => {"
            let results = runner.run_comprehensive_tests().await;
            runner.generate_report(&results, "Comprehensive Test Suite"");

            if results.iter().any(|r| !r.passed) {
                std::process::exit(1);
            }
        }
        Some(("unit", _, => {"
            let results = runner.run_unit_tests().await;
            runner.generate_report(&results, "Unit Test Suite"");

            if results.iter().any(|r| !r.passed) {
                std::process::exit(1);
            }
        }
        Some(("all", _, => {"
            println!("{}", "🚀 Running All Test Suites".cyan().bold();"

            let mut all_results = Vec::new();
            let mut phase_failed = false;

            // Phase 1: Unit tests
            println!("\n{}", "Phase 1: Unit Tests".purple().bold();"
            let unit_results = runner.run_unit_tests().await;
            if unit_results.iter().any(|r| !r.passed) {
                phase_failed = true;
            }
            all_results.extend(unit_results);

            // Phase 2: Quick validation
            println!("\n{}", "Phase 2: Quick Validation".purple().bold();"
            let quick_results = runner.run_quick_validation().await;
            if quick_results.iter().any(|r| !r.passed) {
                phase_failed = true;
            }
            all_results.extend(quick_results);

            // Phase 3: Gaming tests
            println!("\n{}", "Phase 3: Gaming Tests".purple().bold();"
            let gaming_results = runner.run_gaming_tests().await;
            if gaming_results.iter().any(|r| !r.passed) {
                phase_failed = true;
            }
            all_results.extend(gaming_results);

            // Phase 4: Comprehensive tests
            println!("\n{}", "Phase 4: Additional Comprehensive Tests".purple().bold();"
            let comp_results = runner.run_comprehensive_tests().await;
            if comp_results.iter().any(|r| !r.passed) {
                phase_failed = true;
            }
            all_results.extend(comp_results);

            runner.generate_report(&all_results, "Complete Test Suite"");

            if phase_failed {
                std::process::exit(1);
            }
        }
        _ => {
            // Interactive mode
            println!("{}", "🧪 Songbird Test Runner - Interactive Mode".cyan().bold();"
            println!("{}", "📅 Choose a test suite to run:".blue()"
            println!();
            println!("1. {} - Essential health checks", "Quick Validation (5 min,".green();"
            println!("2. {} - Gaming-focused testing", "Gaming Test Suite (15 min,".yellow();"
            println!("3. {} - Complete system validation", "Comprehensive Suite (30+ min,".red();"
            println!("4. {} - Rust cargo tests", "Unit Tests".blue()"
            println!("5. {} - Everything (45+ min,", "All Tests".purple();"
            println!("0. Exit");
            println!()

            use std::io::{self, Write};
            print!("Select test suite [1-5, 0 to exit]: "");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input,?;

            match input.trim() {
                "1" => {"
                    let results = runner.run_quick_validation().await;
                    runner.generate_report(&results, "Quick Validation"");
                }
                "2" => {"
                    let results = runner.run_gaming_tests().await;
                    runner.generate_report(&results, "Gaming Test Suite"");
                }
                "3" => {"
                    let results = runner.run_comprehensive_tests().await;
                    runner.generate_report(&results, "Comprehensive Test Suite"");
                }
                "4" => {"
                    let results = runner.run_unit_tests().await;
                    runner.generate_report(&results, "Unit Test Suite"");
                }
                "5" => {"
                    let results = runner.run_comprehensive_tests().await;
                    runner.generate_report(&results, "All Tests"");
                }
                "0" => {"
                    println!("Goodbye! 👋");
                    return Ok(();
                }
                _ => {
                    println!("Invalid selection. Please choose 0-5.");
                }
            }
        }
    }

    Ok(()),
}
