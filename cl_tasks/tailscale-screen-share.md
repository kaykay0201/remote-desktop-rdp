# Task: Replace IronRDP + Cloudflared with Custom Screen Capture + Tailscale

## Objective
Replace the IronRDP protocol and cloudflared tunneling with a custom screen capture/streaming solution over Tailscale mesh VPN.

## Phases
1. Phase 0: Setup (branch, decouple updater, add deps, create stubs)
2. Parallel Phase: Protocol, Capture, Input, Network modules (4 agents)
3. Phase 3b: Merge + Rewire (delete old modules, rewire app.rs)
4. Phase 4: Polish (heartbeat, reconnection, adaptive quality)

## Files Modified
- Cargo.toml (new deps, remove old)
- src/main.rs (new mod declarations)
- src/updater.rs (decouple from cloudflared)
- New: src/protocol/, src/capture/, src/input/, src/network/, src/tailscale.rs
- Delete: src/rdp/, src/tunnel.rs, src/cloudflared.rs, src/ui/setup.rs

## Risks
- turbojpeg Windows build (fallback: image crate)
- scrap Capturer is !Send (use spawn_blocking)
- iced subscription lifecycle (keep flags true until Stopped)
