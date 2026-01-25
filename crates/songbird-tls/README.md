# songbird-tls

Pure Rust TLS 1.3 implementation for Songbird HTTP client.

## Features

- RFC 8446 compliant TLS 1.3
- Zero C dependencies (Pure Rust)
- BearDog crypto delegation via Tower Atomic pattern
- Stream-based parsing for efficient memory usage

## Status

✅ Production-ready TLS 1.3 client
✅ Tested against cloudflare.com, google.com, github.com
✅ TRUE ecoBin compliant (100% Pure Rust)

## Usage

This crate is internal to songbird-http-client and not intended for standalone use.

See `songbird-http-client` for the public HTTP/HTTPS API.

