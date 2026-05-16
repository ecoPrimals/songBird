#![allow(clippy::unwrap_used, reason = "test assertions")]

use crate::bin_interface::ServerArgs;
use clap::Parser;

#[derive(Parser)]
#[command(name = "songbird")]
struct Cli {
    #[command(flatten)]
    args: ServerArgs,
}

fn effective_external_port(args: &ServerArgs) -> u16 {
    args.federation_port.unwrap_or(args.port)
}

#[test]
fn federation_port_takes_precedence_over_port() {
    let cli =
        Cli::try_parse_from(["songbird", "--port", "8080", "--federation-port", "9090"]).unwrap();
    assert_eq!(cli.args.port, 8080);
    assert_eq!(cli.args.federation_port, Some(9090));
    assert_eq!(effective_external_port(&cli.args), 9090);
}

#[test]
fn port_used_when_federation_port_absent() {
    let cli = Cli::try_parse_from(["songbird", "--port", "7777"]).unwrap();
    assert_eq!(effective_external_port(&cli.args), 7777);
}

#[test]
fn daemon_and_verbose_flags_parse() {
    let cli = Cli::try_parse_from(["songbird", "--port", "80", "--daemon", "--verbose"]).unwrap();
    assert!(cli.args.daemon);
    assert!(cli.args.verbose);
}

#[test]
fn socket_and_listen_optional() {
    let cli = Cli::try_parse_from(["songbird"]).unwrap();
    assert!(cli.args.socket.is_none());
    assert!(cli.args.listen.is_none());
}

#[test]
fn tcp_listen_address_accepts_host_port() {
    let cli =
        Cli::try_parse_from(["songbird", "--listen", "127.0.0.1:9901", "--port", "3000"]).unwrap();
    assert_eq!(cli.args.listen.as_deref(), Some("127.0.0.1:9901"));
}

#[test]
fn dark_forest_flag_parses() {
    let cli = Cli::try_parse_from(["songbird", "--dark-forest"]).unwrap();
    assert!(cli.args.dark_forest);
}

#[test]
fn dark_forest_defaults_to_false() {
    let cli = Cli::try_parse_from(["songbird"]).unwrap();
    assert!(!cli.args.dark_forest);
}

#[test]
fn pid_dir_flag_parses() {
    let cli = Cli::try_parse_from(["songbird", "--pid-dir", "/data/local/tmp"]).unwrap();
    assert_eq!(cli.args.pid_dir.as_deref(), Some("/data/local/tmp"));
}

#[test]
fn pid_dir_defaults_to_none() {
    let cli = Cli::try_parse_from(["songbird"]).unwrap();
    assert!(cli.args.pid_dir.is_none());
}

#[test]
fn bind_flag_defaults_to_localhost() {
    let cli = Cli::try_parse_from(["songbird"]).unwrap();
    assert_eq!(cli.args.bind, "127.0.0.1");
}

#[test]
fn bind_flag_accepts_host() {
    let cli = Cli::try_parse_from(["songbird", "--bind", "0.0.0.0"]).unwrap();
    assert_eq!(cli.args.bind, "0.0.0.0");
}

#[test]
fn bind_flag_accepts_host_port() {
    let cli = Cli::try_parse_from(["songbird", "--bind", "192.168.1.5:9200"]).unwrap();
    assert_eq!(cli.args.bind, "192.168.1.5:9200");
}

#[test]
fn parse_bind_flag_host_only() {
    let (host, port) = super::parse_bind_flag("127.0.0.1");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port, None);
}

#[test]
fn parse_bind_flag_host_port() {
    let (host, port) = super::parse_bind_flag("0.0.0.0:9200");
    assert_eq!(host, "0.0.0.0");
    assert_eq!(port, Some(9200));
}

#[test]
fn parse_bind_flag_ipv6_bracketed_port() {
    let (host, port) = super::parse_bind_flag("[::1]:8080");
    assert_eq!(host, "[::1]");
    assert_eq!(port, Some(8080));
}

#[test]
fn parse_bind_flag_ipv6_no_port() {
    let (host, port) = super::parse_bind_flag("::");
    assert_eq!(host, "::");
    assert_eq!(port, None);
}

#[test]
fn parse_bind_flag_ipv6_bracketed_no_port() {
    let (host, port) = super::parse_bind_flag("[::1]");
    assert_eq!(host, "[::1]");
    assert_eq!(port, None);
}

#[test]
fn all_flags_combined() {
    let cli = Cli::try_parse_from([
        "songbird",
        "--port",
        "9090",
        "--bind",
        "0.0.0.0",
        "--dark-forest",
        "--pid-dir",
        "/run/songbird",
        "--verbose",
        "--daemon",
    ])
    .unwrap();
    assert_eq!(cli.args.port, 9090);
    assert_eq!(cli.args.bind, "0.0.0.0");
    assert!(cli.args.dark_forest);
    assert_eq!(cli.args.pid_dir.as_deref(), Some("/run/songbird"));
    assert!(cli.args.verbose);
    assert!(cli.args.daemon);
}
