// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::types::EnrollPhase;
use serde_json::json;
use tracing::info;

/// Register an SSH public key on Forgejo via its REST API.
///
/// Uses `curl` to POST to the Forgejo API, avoiding additional HTTP client
/// dependencies. Requires `FORGEJO_API_TOKEN` and optionally `FORGEJO_API_URL`.
pub(super) async fn register_forgejo_key(gate_name: &str, ssh_pubkey: &str) -> (EnrollPhase, bool) {
    let forgejo_url = match songbird_process_env::var("FORGEJO_API_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => {
            return (
                EnrollPhase {
                    name: "forgejo.key".into(),
                    ok: false,
                    detail: "FORGEJO_API_URL not set — cannot register SSH key".into(),
                },
                false,
            );
        }
    };
    let forgejo_token = songbird_process_env::var("FORGEJO_API_TOKEN");

    let Ok(token) = forgejo_token else {
        return (
            EnrollPhase {
                name: "forgejo.key".into(),
                ok: false,
                detail: "FORGEJO_API_TOKEN not set — cannot register SSH key".into(),
            },
            false,
        );
    };

    let url = format!("{forgejo_url}/user/keys");
    let body = json!({
        "title": format!("{gate_name}-deploy"),
        "key": ssh_pubkey,
        "read_only": false,
    });

    match tokio::process::Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/stdout",
            "-w",
            "\n%{http_code}",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: token {token}"),
            "-H",
            "Content-Type: application/json",
            "-d",
            &body.to_string(),
            &url,
        ])
        .output()
        .await
    {
        Ok(output) => {
            let raw = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = raw.trim().rsplitn(2, '\n').collect();
            let status_code: u16 = lines.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let response_body = lines.get(1).unwrap_or(&"");

            if (200..300).contains(&status_code) {
                info!(gate = %gate_name, "forgejo.key: SSH key registered");
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: true,
                        detail: format!("SSH key registered as {gate_name}-deploy"),
                    },
                    true,
                )
            } else if status_code == 422 || response_body.contains("already") {
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: true,
                        detail: format!("SSH key already registered (HTTP {status_code})"),
                    },
                    true,
                )
            } else {
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: false,
                        detail: format!("Forgejo API error (HTTP {status_code}): {response_body}"),
                    },
                    false,
                )
            }
        }
        Err(e) => (
            EnrollPhase {
                name: "forgejo.key".into(),
                ok: false,
                detail: format!("curl failed: {e}"),
            },
            false,
        ),
    }
}
