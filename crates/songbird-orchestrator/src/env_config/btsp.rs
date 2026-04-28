// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

/// BTSP guard: refuse to start when both `FAMILY_ID` (non-default) and
/// `BIOMEOS_INSECURE=1` are set.
///
/// Per `BTSP_PROTOCOL_STANDARD.md` v1.0 and `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1:
/// you cannot claim a family AND skip authentication. This is a hard error.
///
/// # Errors
///
/// Returns an error if the conflicting configuration is detected.
pub fn validate_btsp_insecure_guard() -> anyhow::Result<()> {
    validate_btsp_insecure_guard_with(|k| songbird_process_env::var(k))
}

/// Injectable variant for concurrent-safe testing.
pub fn validate_btsp_insecure_guard_with<F>(env_reader: F) -> anyhow::Result<()>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = env_reader("FAMILY_ID")
        .or_else(|_| env_reader("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env_reader("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let insecure = env_reader("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    anyhow::ensure!(
        !(fid != "default" && !fid.is_empty() && insecure),
        "FATAL: FAMILY_ID={fid:?} and BIOMEOS_INSECURE=1 are both set. \
         Per BTSP_PROTOCOL_STANDARD.md v1.0: you cannot claim a family AND skip authentication. \
         Either remove BIOMEOS_INSECURE or unset FAMILY_ID."
    );
    Ok(())
}
