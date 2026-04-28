# Helios-Terminal Architecture Plan

> **Goal:** Turn the Warp open-source terminal into a dedicated shell for running helios-agi (Pi runtime), replacing Warp's AI layer entirely while keeping its world-class terminal UX.

## 1. Product Vision

```
┌──────────────────────────────────────────────────────────┐
│                    Helios-Terminal                         │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Warp Terminal Shell (Rust)                         │  │
│  │  • Block-based UI (warpui/warpui_core)             │  │
│  │  • Terminal emulator (warp_terminal)                │  │
│  │  • Editor, completions, LSP, voice, computer_use   │  │
│  │  • All Warp features preserved                     │  │
│  └──────────────────────┬─────────────────────────────┘  │
│                         │ subprocess / pty                 │
│  ┌──────────────────────▼─────────────────────────────┐  │
│  │  Pi Runtime (helios-agi)                            │  │
│  │  • Helios orchestrator (agent identity)             │  │
│  │  • All skills, extensions, subagents                │  │
│  │  • Provider auth (~/.pi/providers.json)             │  │
│  │  • Memgraph, tools, MCP, governance                 │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌────────────────────────────────────────────────────┐  │
│  │  helios-agi OAuth (optional cloud layer)            │  │
│  │  • Usage tracking, team features, cloud sync        │  │
│  │  • Graceful degradation — works fully offline       │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

**helios-desktop** remains a completely separate Electron product. Features from Warp may be ported to it over time, but the two products are independent.

## 2. What Changes vs. What Stays

### KEEP (all Warp terminal features)
| Crate | Purpose | Action |
|-------|---------|--------|
| `warpui` / `warpui_core` | UI framework (MIT) | Keep as-is |
| `warp_terminal` | Terminal emulator | Keep as-is |
| `editor` | Inline code editing | Keep as-is |
| `warp_completer` | Command autocomplete | Keep as-is |
| `lsp` | Language server protocol | Keep as-is |
| `voice_input` | Voice input | Keep as-is |
| `computer_use` | Computer use | Keep as-is |
| `persistence` | Local storage | Keep as-is |
| `settings` | App settings | Modify (remove Warp-specific, add Helios) |
| `warp_core` | Core app logic | Modify (rebrand, reroute AI) |
| `ipc` | Inter-process comm | Keep as-is |

### REPLACE
| Crate | Purpose | Replacement |
|-------|---------|-------------|
| `ai` (22K lines, 68 files) | Warp's AI agent layer | **Helios Bridge** — thin Rust crate that spawns `pi` as subprocess |
| `firebase` | Auth via Firebase | **helios_auth** — hybrid: Pi local + helios-agi OAuth |
| `graphql` / `warp_graphql_schema` | Warp API schema | Strip or replace with helios-agi API |
| `warp_server_client` | Warp cloud backend | Replace with helios-agi backend |
| `onboarding` | Warp onboarding | Replace with Helios-Terminal onboarding |

### STRIP (remove entirely)
- Warp Drive (cloud-dependent on Warp infra)
- `managed_secrets` / `managed_secrets_wasm` (Warp-specific secret management)
- Firebase auth flows
- Warp telemetry endpoints
- Oz agent orchestration hooks

## 3. Helios Bridge Architecture (`crates/helios_bridge`)

The core integration point. Replaces `crates/ai` with a thin Rust crate.

```rust
// crates/helios_bridge/src/lib.rs
//
// HeliosBridge: Spawns and manages the `pi` CLI process
// All agent intelligence lives in Pi runtime — this crate is just plumbing.

pub struct HeliosBridge {
    pi_process: Option<Child>,
    config: HeliosConfig,
}

pub struct HeliosConfig {
    /// Path to `pi` binary (default: auto-detect from PATH)
    pub pi_binary: PathBuf,
    /// Default agent to use (default: "helios")
    pub default_agent: String,
    /// Pi home directory (default: ~/.pi)
    pub pi_home: PathBuf,
    /// Whether to use helios-agi cloud auth
    pub cloud_auth: bool,
}

impl HeliosBridge {
    /// Spawn a new Pi session in the terminal
    pub async fn spawn_session(&mut self, cwd: &Path) -> Result<PiSession> {
        let child = Command::new(&self.config.pi_binary)
            .args(["--agent", &self.config.default_agent])
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        self.pi_process = Some(child);
        Ok(PiSession { /* ... */ })
    }
    
    /// Check if Pi runtime is available
    pub fn is_pi_available() -> bool {
        which::which("pi").is_ok()
    }
    
    /// Get Pi version
    pub fn pi_version() -> Option<String> {
        Command::new("pi").arg("--version").output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
    }
}
```

### Integration with Warp's Agent UI

Warp has an "agent panel" UI that displays agent actions, file edits, citations, etc. Instead of gutting this entirely, we bridge it:

```
Warp Agent UI ←→ HeliosBridge ←→ Pi stdout/stderr (parsed)
                      │
                      ├── Parse Pi's structured output (tool calls, diffs, etc.)
                      ├── Forward to Warp's existing action_result types
                      └── Display in Warp's block-based UI
```

Pi already outputs structured content. The bridge parses Pi's terminal output and maps it to Warp's `AIAgentActionResultType` enum for native display.

## 4. Auth Architecture

### Layer 1: Pi Local Auth (offline-capable)
```
~/.pi/providers.json → API keys for Anthropic, OpenAI, Google, etc.
Pi runtime manages: sessions, tokens, rate limits, model switching
```

### Layer 2: helios-agi Cloud Auth (optional)
```
crates/helios_auth/
├── oauth.rs          — OAuth2 PKCE flow → helios-agi.dev
├── token_manager.rs  — JWT refresh, storage in keychain
├── cloud_features.rs — What cloud auth unlocks:
│   ├── Usage analytics
│   ├── Team/org management
│   ├── Settings sync across devices
│   └── Shared skills/extensions marketplace
└── fallback.rs       — Graceful degradation when offline
```

Replace `crates/firebase` with `crates/helios_auth`. The auth manager in `app/src/auth/` gets rewired to:
1. Check for Pi installation and provider config → always works
2. Optionally prompt for helios-agi cloud login → enhanced features

## 5. Dependency Surgery Map

### `crates/ai` consumers (what needs rewiring)

| File | Import | Bridge Strategy |
|------|--------|-----------------|
| `app/src/lib.rs` | `AIAgentActionResultType`, `FileEdit`, etc. | Re-export from `helios_bridge` with compatible types |
| `app/src/settings/mod.rs` | `pub use ai::*` | Replace with `pub use helios_bridge::settings::*` |
| `app/src/auth/` | `CodebaseIndexManager` | Move to `helios_bridge::indexing` or delegate to Pi |
| `app/src/terminal/input/models/` | `ApiKeyManager` | Replace with Pi's provider config |
| `app/src/terminal/cli_agent.rs` | `SkillProvider` | Map to Pi skills |
| `app/src/code/` | `DiffType`, `DiffDelta` | Keep as shared types in `helios_bridge` |
| `crates/onboarding/` | `ai` crate | Rewrite onboarding for Helios-Terminal |

### External API dependency
- `warp_multi_agent_api` (protobuf from `warpdotdev/warp-proto-apis`) — this is Warp's cloud agent protocol
- **Action:** Strip entirely. Pi has its own protocol.

## 6. Execution Plan (Phases)

### Phase 1: Fork Setup & Branding (Today)
- [x] Fork warpdotdev/warp → helios-agi/warp
- [x] Clone as helios-terminal
- [ ] Update branding: README, app name, about dialog
- [ ] Set up .planning/ structure
- [ ] Verify build works as-is (`./script/bootstrap && ./script/run`)

### Phase 2: Create helios_bridge Crate (Week 1)
- [ ] Create `crates/helios_bridge/` with Pi subprocess management
- [ ] Define bridge types compatible with existing `ai` crate interfaces
- [ ] Implement structured output parser for Pi terminal output
- [ ] Create feature flag: `--features helios` vs `--features warp-ai`

### Phase 3: Auth Replacement (Week 1-2)
- [ ] Create `crates/helios_auth/` 
- [ ] Implement Pi provider detection (`~/.pi/providers.json`)
- [ ] Stub helios-agi OAuth flow (full implementation later)
- [ ] Rewire `app/src/auth/` to use helios_auth

### Phase 4: AI Layer Swap (Week 2-3)
- [ ] Rewire all `use ai::` imports in `app/` to `helios_bridge`
- [ ] Remove `warp_multi_agent_api` dependency
- [ ] Update Warp's agent panel UI to display Pi output
- [ ] Remove Warp-specific agent features (Oz, cloud agents)

### Phase 5: Strip Warp Cloud Dependencies (Week 3)
- [ ] Remove `crates/graphql` / `warp_graphql_schema`
- [ ] Remove `crates/warp_server_client`
- [ ] Remove Firebase SDK
- [ ] Remove Warp telemetry endpoints
- [ ] Remove `managed_secrets` / `managed_secrets_wasm`

### Phase 6: Polish & Ship (Week 4)
- [ ] Full branding pass (icons, splash, about)
- [ ] Onboarding flow for Helios-Terminal
- [ ] Pi installation check + guided setup
- [ ] Build pipeline (macOS, Linux, Windows)
- [ ] Release as helios-agi/helios-terminal

## 7. helios-desktop Feature Mining

Features from Warp to evaluate for helios-desktop backport:
- **Block-based terminal rendering** → Could improve helios-desktop's terminal panels
- **Command completions engine** → Richer autocomplete in helios-desktop
- **LSP integration pattern** → Language awareness in helios-desktop editor
- **Computer use crate** → Desktop automation capabilities
- **Voice input** → Voice commands in helios-desktop
- **warpui layout system** → UI patterns for helios-desktop (though it's Electron/React)

## 8. License Compliance

- `warpui` / `warpui_core` → MIT ✅ (can use freely)
- Everything else → AGPL v3 (must open-source modifications, which we're doing via helios-agi/warp fork)
- Must retain AGPL notice and attribution
- Our additions (helios_bridge, helios_auth) can be any compatible license

## 9. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Build complexity (Rust monorepo) | High | Phase 1 validates build before changes |
| `ai` crate tentacles deep in app | High | Feature flag approach allows gradual migration |
| `warp_multi_agent_api` protobuf dependency | Medium | Clean cut — Pi doesn't need it |
| Platform-specific builds | Medium | Follow Warp's existing CI/CD |
| Pi subprocess lifecycle management | Medium | Well-understood pattern (like VSCode + extensions) |
| Upstream Warp changes | Low | Hard fork allows selective cherry-picking |
