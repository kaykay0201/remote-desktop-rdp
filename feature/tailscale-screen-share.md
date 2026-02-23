# Feature: Replace IronRDP + Cloudflared with Screen Capture + Tailscale

## Completion Date
2026-02-23

## Summary
Replaced the IronRDP RDP protocol and cloudflared tunnel proxy with a custom screen capture/streaming solution over Tailscale mesh VPN.

## What Was Implemented

### New Modules
- `src/protocol/` — Wire protocol with bincode v2 serialization, LZ4 compression, tokio-util codec
- `src/capture/` — Screen capture via scrap, JPEG encoding via image crate, LZ4 compression
- `src/input_handler/` — Bidirectional input: iced→protocol translation (access side), enigo simulation (host side)
- `src/network/` — TCP server (host) and client (access) subscriptions using Framed<TcpStream, MessageCodec>
- `src/tailscale.rs` — Tailscale status check via `tailscale status --json`

### Removed Modules
- `src/rdp/` — IronRDP protocol (connection, session, input)
- `src/tunnel.rs` — Cloudflared tunnel management
- `src/cloudflared.rs` — Cloudflared download/management
- `src/ui/setup.rs` — Cloudflared setup screen

### Modified Files
- `Cargo.toml` — Removed ironrdp, ironrdp-tokio, tokio-native-tls, native-tls, smallvec, which. Added bincode, lz4_flex, tokio-util, bytes, scrap, image, enigo, serde_json.
- `src/main.rs` — Updated module declarations
- `src/app.rs` — Complete rewire: new Message/Screen enums, Tailscale-based networking
- `src/error.rs` — Renamed RdpError → AppError, updated variants
- `src/config/profile.rs` — Simplified: host_ip, port, display_name (removed hostname, username, password, width, height)
- `src/ui/login.rs` — Tailscale IP + port input (removed tunnel URL, credentials, resolution)
- `src/ui/host.rs` — Shows Tailscale address instead of tunnel URL
- `src/ui/viewer.rs` — Removed RdpConnection dependency, uses ConnectionHandle
- `src/ui/mode_select.rs` — Updated text labels
- `src/ui/mod.rs` — Removed setup module
- `src/updater.rs` — Decoupled from cloudflared (local app_data_dir)

## Test Results
67 tests passing (was 60 before migration)

## Follow-up Items
- Phase 4: Heartbeat ping/pong, reconnection, adaptive quality, FPS counter
- Wire capture_loop into network server for actual frame streaming
- Wire InputHandler into network server for input simulation on host
