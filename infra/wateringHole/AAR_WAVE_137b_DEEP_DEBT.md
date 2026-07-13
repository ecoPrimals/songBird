# AAR — Wave 137b Deep Debt Evolution

**Primal**: songBird  
**Date**: July 13, 2026  
**Gate**: flockGate → origin/main + forgejo/main  
**Wave**: 137b (closing)

---

## Delivered (6 commits)

| Commit | Type | Summary |
|--------|------|---------|
| `74cf710` | feat | HTTPS outbound proxy via `tokio-rustls` + system CA certs (FP-API complete) |
| `a3c2871` | refactor | Per-request allocation elimination in drawbridge proxy hot path |
| `73b0c7d` | feat | **DRAWBRIDGE-CAP**: Runtime capabilities in `capabilities.list`; `capability.call` drawbridge fallback; `capability.resolve` for proxy-router caps |
| `0b042af` | refactor | Drawbridge header parsing simplification, dead constant removal (upstream) |
| `718d18d` | refactor | `to_lowercase()` elimination across 17 files / 5 crates; centralized `parse_bool_relaxed()` |
| `6463876` | refactor | Remaining hot-path `to_lowercase()` in drawbridge + http-client domain matching |

---

## Key Decisions

### DRAWBRIDGE-CAP Architecture

**Problem**: `capabilities.list` returned only 15 hardcoded native capabilities. Drawbridge-served services (jupyter, inference, GIS proxy) were invisible to `capability.call` routing.

**Solution (3-layer)**:
1. `capabilities_list_with_runtime()` — merges runtime caps from `SONGBIRD_PROXY_ROUTES` + `SONGBIRD_DRAWBRIDGE_ROUTES` env vars into Wire Standard L3 envelope. Internal caps (prefixed `_`) filtered.
2. `capability.call` fallback — when no UDS provider registered, checks proxy router for HTTP backend route.
3. `capability.resolve` — returns drawbridge caps as `TransportEndpoint::Tcp` (local HTTP service).

**Trade-off**: Re-reads `DrawbridgeConfig::from_env()` on each `capabilities.list` call rather than caching. Acceptable because introspection calls are infrequent vs. data-plane proxy calls. Avoids stale state.

### Allocation Elimination Strategy

**Problem**: `to_lowercase()` on every comparison allocates a fresh `String`. In hot paths (header parsing, domain matching, env bool parsing), this adds GC pressure per-request.

**Solution**: 
- `eq_ignore_ascii_case()` for equality checks (zero-alloc)
- `[u8]::eq_ignore_ascii_case()` for substring/suffix domain matching (zero-alloc byte windows)
- `parse_bool_relaxed()` centralized in `songbird_types::error_helpers` — single canonical implementation replaces 6 duplicated inline patterns

**Impact**: 17 files across `songbird-types`, `songbird-config`, `songbird-http-client`, `songbird-orchestrator`, `songbird-universal-ipc`. All `FromStr` impls for config enums now allocation-free.

### FP-API-CADDY Snippet

Drafted `infra/caddy/footprint-gis-proxy.Caddyfile` (reusable named snippet `(footprint_gis_proxy)`) for 10 GIS upstream hosts. Caddy handles upstream TLS natively — bypasses SOCKET-DIR-UNIFY blocker. Deployed to golgi by sporeGate team same day.

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| `to_lowercase()` in hot paths | ~55 sites | 0 in per-request/connection code |
| `drawbridge.rs` production LOC | 788 | 685 (upstream cleanup absorbed) |
| `capabilities.list` cap count | 15 (static) | 15 + runtime (dynamic) |
| Clippy warnings | 0 | 0 |
| Test failures | 0 | 0 |

---

## Remaining for songBird (Wave 138+)

- **DRAWBRIDGE-CAP verified on sporeGate**: Needs deployment of new binary (`6463876`) to sporeGate for upstream verification that `capabilities.list` now returns drawbridge caps.
- **Coverage push**: 73.41% → target 90%. Primarily I/O-heavy modules need mock infra.
- **Deep debt (next tier)**: `"literal".to_string()` → `String::from()` migration (~100 sites in http-client, discovery, CLI). Lower priority — stylistic, not perf.
- **VERSION-SKEW discussion**: 3 version ranges need harmonization strategy (cross-team).

---

## Blockers (none from songBird)

songBird is not blocking any team. All provided capabilities live. FP-API GIS proxy E2E verified. Drawbridge weak bond pattern operational.

---

*AAR complete. songBird Wave 137b: 6 commits, 0 regressions, 0 blockers introduced.*
