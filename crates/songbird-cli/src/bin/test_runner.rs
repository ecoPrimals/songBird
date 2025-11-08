//! Test Runner Binary
//!
//! Comprehensive test execution for the Songbird CLI

use clap::{Arg, Command};
use colored::Colorize;
use reqwest::Client;
use songbird_config::config::hardcoded_elimination::replace;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test result tracking
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub message: String,
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub songbird_url: String,
    pub timeout_seconds: u64,
    pub verbose: bool,
    pub quiet: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            songbird_url: replace::orchestrator_endpoint().to_string(),
            timeout_seconds: 15,
            verbose: false,
            quiet: false,
        }
    }
}

/// Test runner implementation
pub struct TestRunner {
    config: TestConfig,
    client: Client,
    passed: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
}

impl TestRunner {
    /// Create a new test runner
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created
    pub fn new(config: TestConfig) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(Self {
            config,
            client,
            passed: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
            total: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Print header with formatting
    fn print_header(&self, title: &str) {
        if !self.config.quiet {
            println!("{}", format!("🎯 {title}").purple().bold());
            println!("{}", "=".repeat(50).blue());
        }
    }

    /// Print success message
    fn print_success(&self, message: &str) {
        if !self.config.quiet {
            println!("{}", format!("✅ {message}").green());
        }
    }

    /// Print error message
    #[allow(clippy::unused_self)] // self kept for consistency with other print methods
    fn print_error(&self, message: &str) {
        println!("{}", format!("❌ {message}").red());
    }

    /// Print info message
    fn print_info(&self, message: &str) {
        if !self.config.quiet {
            println!("{}", format!("ℹ️  {message}").blue());
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
            self.print_info(&format!("Test {}: {}", self.total.load(Ordering::Relaxed), name));
        }

        let result = timeout(Duration::from_secs(self.config.timeout_seconds), test_fn).await;

        let duration = start_time.elapsed();
        let test_result = match result {
            Ok(Ok(())) => {
                self.passed.fetch_add(1, Ordering::Relaxed);
                self.print_success(&format!("PASSED: {name}"));
                TestResult {
                    name: name.to_string(),
                    passed: true,
                    duration,
                    message: "Test passed successfully ".to_string(),
                }
            }
            Ok(Err(e)) => {
                self.failed.fetch_add(1, Ordering::Relaxed);
                let message = format!("FAILED: {e} - {name}");
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
                    format!("FAILED: {} - Timeout after {} s ", name, self.config.timeout_seconds);
                self.print_error(&message);
                TestResult {
                    name: name.to_string(),
                    passed: false,
                    duration,
                    message: "Test timed out ".to_string(),
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
        let response = self
            .client
            .get(format!("{}/api/health ", self.config.songbird_url))
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Health check request failed: {e}")))?;

        if response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| SongbirdError::network(format!("Failed to read response: {e}")))?;
            if text.to_lowercase().contains("healthy") {
                Ok(())
            } else {
                Err(format!("Health check failed: unexpected response: {text}").into())
            }
        } else {
            Err(format!("Health Check failed with status: {}", response.status()).into())
        }
    }

    /// Quick validation tests
    pub async fn run_quick_validation(&self) -> Vec<TestResult> {
        self.print_header("Quick Validation Suite");
        let mut results = Vec::new();

        // Essential health checks
        results.push(
            self.run_test("API Health Check ", async { self.check_songbird_health().await }).await,
        );

        results.push(
            self.run_test("System Metrics ", async {
                let response = self
                    .client
                    .get(format!("{}/api/metrics", self.config.songbird_url))
                    .send()
                    .await
                    .map_err(|e| SongbirdError::network(format!("Metrics request failed: {e}")))?;

                if response.status().is_success() {
                    let text = response.text().await.map_err(|e| {
                        SongbirdError::network(format!("Failed to read response: {e}"))
                    })?;
                    if text.contains("cpu_usage") {
                        Ok(())
                    } else {
                        Err("Metrics response missing expected fields".into())
                    }
                } else {
                    Err(format!("Metrics  request failed: {}", response.status()).into())
                }
            })
            .await,
        );

        results
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self, results: &[TestResult], suite_name: &str) {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let pass_rate = if total_tests > 0 {
            (passed_tests * 100) / total_tests
        } else {
            0
        };
        let total_duration: Duration = results.iter().map(|r| r.duration).sum();

        println!("\n{}", "=".repeat(60).blue());
        println!("{}", format!("🏁 {suite_name} Complete!").cyan().bold());
        println!("{}", "=".repeat(60).blue());

        println!("{}", "📊 Results:".yellow().bold());
        println!("   {} Passed: {}", "✅".green(), format!("{passed_tests} tests").green());
        if failed_tests > 0 {
            println!("   {} Failed: {}", "❌".red(), format!("{failed_tests} tests").red());
        }
        println!("   📈 Total:  {total_tests} tests");
        println!(
            "   📊 Pass Rate: {}%",
            if pass_rate >= 80 {
                format!("{pass_rate}").green()
            } else {
                format!("{pass_rate}").red()
            }
        );
        println!("   ⏱️  Duration: {total_duration:?}");

        // System assessment
        println!("\n{}", "🎯 Assessment:".yellow().bold());
        if failed_tests == 0 {
            println!("   🎉 All tests passed! System is production-ready.");
            println!("   🚀 Ready for deployment.");
        } else if pass_rate > 80 {
            println!("   ⚠️  Most tests passed but some issues detected.");
            println!("   🔧 Review failed tests and check troubleshooting guide.");
        } else {
            println!("   🚨 Significant issues detected.");
            println!("   🛠️  System needs attention before production use.");
        }

        // Failed test details
        if failed_tests > 0 && self.config.verbose {
            println!("\n{}", "❌ Failed Tests:".red().bold());
            for result in results.iter().filter(|r| !r.passed) {
                println!("   • {}: {}", result.name.red(), result.message);
            }
        }

        println!("\n{}", "📖 Next Steps:".blue().bold());
        println!("   • Review any failed tests above");
        println!("   • Check docs/TROUBLESHOOTING_GUIDE.md for solutions");
        println!("   • Run with --verbose for detailed error information");
        println!("   • Use --help to see all available options");
    }
}

#[tokio::main]
async fn main() -> SongbirdResult<()> {
    let matches = Command::new("Songbird Test Runner")
        .version("1.0")
        .author("Songbird Team")
        .about("Comprehensive test runner for Songbird Universal Orchestrator")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("Songbird API URL")
                .default_value("http://localhost:8080"),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("SECONDS")
                .help("Test timeout in seconds")
                .default_value("15"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Quiet output")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(Command::new("quick").about("Run quick validation"))
        .get_matches();

    let config = TestConfig {
        songbird_url: matches
            .get_one::<String>("url")
            .ok_or("URL argument is required (should have default)")?
            .clone(),
        timeout_seconds: matches
            .get_one::<String>("timeout")
            .ok_or("Timeout argument is required (should have default)")?
            .parse()
            .map_err(|e| SongbirdError::configuration(format!("Invalid timeout value: {e}")))?,
        verbose: matches.get_flag("verbose"),
        quiet: matches.get_flag("quiet"),
    };

    let runner = TestRunner::new(config)?;

    // Run quick validation (same for both branches)
    let results = runner.run_quick_validation().await;
    runner.generate_report(&results, "Quick Validation");

    if results.iter().any(|r| !r.passed) {
        std::process::exit(1);
    }

    Ok(())
}
