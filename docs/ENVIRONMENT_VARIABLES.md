# Songbird Environment Variables

**Last Updated**: May 12, 2026
**Version**: v0.2.1

---

## Security Provider Socket Discovery

Songbird discovers its security provider (BearDog) via the following chain,
tried in order. The first match wins.

### CLI Flag

| Flag | Alias | Effect |
|------|-------|--------|
| `--security-socket <PATH>` | `--beardog-socket` | Overrides all env/filesystem discovery |

### Environment Variables (priority order)

| Variable | Example | Notes |
|----------|---------|-------|
| `SECURITY_PROVIDER_SOCKET` | `/run/user/1000/biomeos/security.sock` | Canonical — preferred for new deployments |
| `SECURITY_SOCKET` | `/run/user/1000/biomeos/security.sock` | Shorthand alias |
| `CRYPTO_PROVIDER_SOCKET` | `/run/user/1000/biomeos/crypto.sock` | Cross-capability alias |
| `BEARDOG_SOCKET` | `/run/user/1000/biomeos/beardog.sock` | **Deprecated** — emits `tracing::warn!` |

### XDG Filesystem Probing (when no env var is set)

All paths below are under `$XDG_RUNTIME_DIR/biomeos/`. If `FAMILY_ID` is set
(e.g. `nucleus01`), family-scoped variants are checked.

| Priority | Path | When |
|----------|------|------|
| 1 | `security.sock` | Capability symlink (wateringHole v1.2 pattern) |
| 2 | `security-{FAMILY_ID}.sock` | Family-scoped security socket |
| 3 | `crypto-{FAMILY_ID}.sock` | Family-scoped crypto alias |
| 4 | `beardog-{FAMILY_ID}.sock` | **Legacy** family-scoped BearDog socket (warns) |

### Temp-dir Fallback

If XDG probing fails:

| Priority | Path | Notes |
|----------|------|-------|
| 5 | `$TMPDIR/biomeos/security.sock` | Last-resort XDG-less fallback |
| 6 | `$TMPDIR/security-provider.sock` | Legacy flat path (warns) |

### Operator Quick Reference

For a standard biomeOS deployment with `FAMILY_ID=nucleus01`:

```bash
# Preferred — explicit
export SECURITY_PROVIDER_SOCKET=/run/user/1000/biomeos/security-nucleus01.sock

# Or let Songbird auto-discover (BearDog binds to the XDG path)
# Songbird finds: $XDG_RUNTIME_DIR/biomeos/security-nucleus01.sock
```

No symlink workarounds are needed as of Wave 173.

---

## Core Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_ENV` | `development` | Runtime environment label |
| `FAMILY_ID` | *(none)* | Multi-family identifier (e.g. `nucleus01`); enables BTSP, family-scoped sockets, and **cryptographic signing of `ipc.register` payloads** (Ed25519 via BearDog delegation) |
| `FAMILY_SEED` | *(none)* | BTSP handshake seed (raw string; Songbird base64-encodes before sending to BearDog) |

## Network

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_HTTP_PORT` | `8080` | HTTP API port |
| `SONGBIRD_HTTPS_PORT` | `8443` | HTTPS API port |
| `SONGBIRD_TARPC_PORT` | `8091` | tarpc RPC port |
| `SONGBIRD_BIND_ADDRESS` | `127.0.0.1` | HTTP server bind address (CLI: `--bind`). Use `0.0.0.0` for LAN exposure |
| `SONGBIRD_DISCOVERY_PORT` | `8081` | Discovery service HTTP port |
| `SONGBIRD_STUN_PORT` | `3478` | STUN server port |
| `SONGBIRD_RELAY_PORT` | `3479` | Relay service port |
| `SONGBIRD_PORT` | `3492` | Canonical service/IGD port |
| `SONGBIRD_MULTICAST_ADDRESS` | `239.255.42.99` | Discovery multicast group |
| `SONGBIRD_CORS_ORIGINS` | `http://localhost:3000` | Comma-separated CORS origins |
| `COMPUTE_HOST` | *(none)* | Compute provider host override |
| `SERVICE_HOST` | *(none)* | Service host override |

## IPC / Socket Paths

| Variable | Default | Description |
|----------|---------|-------------|
| `XDG_RUNTIME_DIR` | `/run/user/{uid}` | XDG base for socket discovery |
| `NEURAL_API_SOCKET` | *(auto-discovered)* | Neural API (biomeOS) socket path |
| `BIOMEOS_INSECURE` | *(unset)* | If set, disables BTSP requirement |

## Capability Endpoints

| Variable | Fallback | Description |
|----------|----------|-------------|
| `COMPUTE_PROVIDER_ENDPOINT` | `COMPUTE_ENDPOINT` → `TOADSTOOL_ENDPOINT` (deprecated) | Compute provider URL |
| `STORAGE_PROVIDER_ENDPOINT` | `STORAGE_ENDPOINT` → `NESTGATE_ENDPOINT` (deprecated) | Storage provider URL |
| `SECURITY_PROVIDER_ENDPOINT` | `SECURITY_ENDPOINT` → `BEARDOG_ENDPOINT` (deprecated) | Security provider URL (HTTP) |
| `AI_PROVIDER_ENDPOINT` | `AI_ENDPOINT` → `SQUIRREL_ENDPOINT` (deprecated) | AI provider URL |

Each capability follows the same fallback chain: `{CAP}_PROVIDER_ENDPOINT` →
`{CAP}_ENDPOINT` → legacy primal-named env var → runtime discovery → error.

## Authorization (MethodGate JH-0)

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_AUTH_MODE` | `permissive` | Method gate enforcement mode. `permissive` logs protected-method calls but allows them; `enforced` rejects unauthenticated calls to protected methods with `-32001 PERMISSION_DENIED` |

The `auth.*` introspection methods (`auth.check`, `auth.mode`, `auth.peer_info`) are
exposed on **all transports** — UDS, TCP NDJSON, and BTSP-encrypted TCP. They are
classified as Public and are always accessible regardless of enforcement mode.

## Dynamic DNS (H2-15)

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_DDNS_ENABLED` | (unset = disabled) | Enable DDNS updates on IP change |
| `SONGBIRD_DDNS_PROVIDER` | `noop` | Provider name (`noop`, `rfc2136`, `cloudflare`) |
| `SONGBIRD_DDNS_HOSTNAME` | — | FQDN to update when public IP changes |
| `SONGBIRD_DDNS_TTL` | `60` | DNS record TTL in seconds |
| `SONGBIRD_DDNS_ZONE` | — | DNS zone (for RFC 2136) |
| `SONGBIRD_DDNS_SERVER` | — | DNS server address (for RFC 2136) |
| `SONGBIRD_DDNS_KEY_NAME` | — | TSIG key name (for RFC 2136) |
| `SONGBIRD_DDNS_KEY_SECRET` | — | TSIG key secret (for RFC 2136) |

### Cloudflare DDNS Provider (H2-15)

| Variable | Description |
|----------|-------------|
| `SONGBIRD_CF_API_TOKEN` | Cloudflare API Bearer token |
| `SONGBIRD_CF_ZONE_ID` | Cloudflare Zone ID for the target DNS zone |

Set `SONGBIRD_DDNS_PROVIDER=cloudflare` and provide the above variables to enable
Cloudflare DNS updates. The `CloudflareDdnsProvider` updates A/AAAA records via
the Cloudflare API v4. Configuration is in `songbird-types::config::ddns` (trait)
and `songbird-stun::ddns_cloudflare` (Cloudflare implementation).
`NoopDdnsProvider` is the default when DDNS is disabled.

## Test / Development

| Variable | Description |
|----------|-------------|
| `SONGBIRD_DISCOVERY_SIMULATION` | If `true`, discovery returns simulated nodes |
| `SONGBIRD_ROUTE_DETECT_ADDR` | Override for UDP route detection probe target (default: `192.0.2.1:80`) |
| `RUST_LOG` | Standard `tracing` filter (e.g. `songbird_orchestrator::ipc=debug`) |
