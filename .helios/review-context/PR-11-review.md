# Helios PR Review — PR #11: WebView Pane Type

**Repo:** helios-agi/warp  
**PR:** #11 — feat: WebView pane type — Inbox, CRM, Interviews in native tabs  
**Size:** XL (2,661 additions, 22 files)  
**Review Pipeline:** 8 domain agents → adversarial verification → dedup/ranking  
**Status:** MERGED (reviewed post-merge)

---

## Verdict: SHIP_WITH_NOTES (post-merge follow-up required)

The PR successfully establishes the WebView pane infrastructure. The ObjC→FFI→Rust→Pane wiring follows established codebase patterns and compiles cleanly. However, the feature has significant gaps that should be tracked as fast-follow issues.

---

## Verified Findings: 14 (deduplicated from 79 raw across 8 domains)

### 🔴 CRITICAL (2)

| ID | Finding | Flagged By |
|----|---------|-----------|
| **V-001** | **Global IPC singleton — multi-pane broken, callback never set** — `static _handler` and `_ipc_callback` are overwritten per `create()` call. `set_ipc_callback()` never called from Rust. All IPC messages silently dropped. | security, logic, architecture, performance, piping |
| **V-002** | **Unrestricted Cypher query execution via IPC** — HTML pages send raw Cypher to Memgraph via postMessage. Any XSS/CDN compromise = full graph read/write/delete. | security |

### 🟠 HIGH (6)

| ID | Finding | Flagged By |
|----|---------|-----------|
| **V-003** | `/tmp` fallback for HTML resources — world-writable directory | security, logic |
| **V-004** | No frame update after layout — WebView frozen at 800×600 at origin | logic, architecture, performance |
| **V-005** | Zero tests for entire WebView feature (4 Rust files, 3 HTML, 3 actions) | test |
| **V-006** | Tailwind CDN loaded over network on every tab open (~400KB, breaks offline) | performance, style |
| **V-007** | `file://` URL allows directory read access via `allowingReadAccessToDirectory:` | security |
| **V-008** | `.planning/` files committed into PR (violates AGENTS.md) | piping |

### 🟡 MEDIUM (5)

| ID | Finding | Flagged By |
|----|---------|-----------|
| **V-009** | `focus_contents()` is a no-op — keyboard dead in WebView tabs | logic |
| **V-010** | `interview.html` orphaned — no `OpenInterview` action | piping |
| **V-011** | ObjC naming convention mismatch (`helios_*` vs `warp_*` prefix) | style |
| **V-012** | SQLite `save_pane_state` has dead WebView arm (latent hazard) | logic, compat |
| **V-013** | No shared `WKProcessPool` — each tab spawns a new process (~50-80MB) | performance |

### 🔵 LOW (1)

| ID | Finding | Flagged By |
|----|---------|-----------|
| **V-014** | No Content Security Policy in HTML pages | security, style |

---

## Cross-Domain Heat Map

| Domain | Critical | High | Medium | Low | Total |
|--------|----------|------|--------|-----|-------|
| Security | 2 | 2 | 0 | 1 | **5** |
| Logic | 1 | 1 | 2 | 0 | **4** |
| Architecture | 1 | 1 | 0 | 0 | **2** |
| Performance | 0 | 2 | 1 | 0 | **3** |
| Piping | 0 | 1 | 1 | 0 | **2** |
| Test | 0 | 1 | 0 | 0 | **1** |
| Style | 0 | 0 | 1 | 0 | **1** |
| Compat | 0 | 0 | 1 | 0 | **1** |

**Highest risk clusters:** ObjC FFI Layer (V-001, V-002, V-007, V-013) and HTML Content (V-002, V-006, V-014)

---

## Recommended Follow-up Issues

### P0 — Before using WebView tabs in production
1. **V-001**: Refactor IPC to per-instance handler with context pointer (wry pattern)
2. **V-002**: Replace raw Cypher with named operation templates
3. **V-003**: Remove `/tmp` fallback, return error page instead

### P1 — Next sprint
4. **V-004**: Hook into layout system for proper WebView frame positioning
5. **V-005**: Add unit tests (snapshot, URL construction, NUL handling)
6. **V-006**: Bundle Tailwind CSS locally
7. **V-007**: Lock `allowingReadAccessToDirectory` to webviews/ dir only
8. **V-008**: Add `.planning/` to `.gitignore`

### P2 — Polish
9. **V-009**: Wire `makeFirstResponder:` for keyboard focus
10. **V-010**: Wire `OpenInterview` action or remove `interview.html`
11. **V-011**: Rename to `warp_webview_*` convention
12. **V-013**: Share `WKProcessPool` across instances

---

## What Went Right
- Clean pane architecture — follows established patterns exactly
- Proper MRC memory management (release in Drop, message handler cleanup)
- Bundle-relative path lookup with dev fallback
- `is_persisted = false` — correct defensive choice until schema migration exists
- `#[cfg(target_os = "macos")]` guards in app-level code
- All WKWebView APIs compatible with deployment target (10.14)

## Review Pipeline Stats
- **Raw findings:** 79 across 8 domain agents
- **After dedup/verification:** 14 verified findings
- **False positive rate:** ~12% (mostly severity downgrades, not invalid claims)
- **Highest-signal domains:** Security (5 verified), Logic (4 verified)
