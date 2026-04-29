# Helios PR Review — PRs #13, #14, #15: Settings Overhaul + Wiring + Upgrade

**Repo:** helios-agi/warp  
**PRs:** #13 (branding), #14 (live data wiring), #15 (upgrade script)  
**Combined:** 765 additions, 183 deletions, 12 files  
**Pipeline:** 7 domain agents → adversarial verification → dedup/ranking  
**Status:** MERGED (reviewed post-merge)

---

## Verdict: SHIP_WITH_NOTES — 3 P0 issues for immediate follow-up

---

## Verified Findings: 11 (deduplicated from 57 raw)

### 🔴 CRITICAL (1)

| ID | Finding | Domains |
|----|---------|---------|
| **V-001** | **docker exec in render path** — spawns subprocess synchronously every render, 28ms+ per call, no timeout, hangs if Docker unavailable. Combined with home dir scan = 99ms blocking render. | perf, arch, logic, security |

### 🟠 HIGH (6)

| ID | Finding | Domains |
|----|---------|---------|
| **V-002** | All 4 data loaders uncached — read disk on every render() call | perf, arch |
| **V-003** | 13 user-visible "Warp" strings remain unrebranded | piping |
| **V-004** | Upgrade script BUNDLE_PATH wrong — cargo bundle output at wrong path | logic |
| **V-005** | `helios-terminal.dev` domain is NXDOMAIN — squattable | security |
| **V-006** | YAML frontmatter parser end-marker bug — matches mid-block `---` | logic |
| **V-007** | Zero tests for 4 new functions + 1 bash script | test |

### 🟡 MEDIUM (4)

| ID | Finding | Domains |
|----|---------|---------|
| **V-008** | Upgrade script defaults to debug build (10-20x slower) | logic, perf |
| **V-009** | git pull without signature verification in upgrade | security |
| **V-010** | Dead `sign_up_button` field produces warning | piping |
| **V-011** | Upgrade script should live in helios-team-installer | arch |

---

## P0 — Fix Immediately

1. **V-001**: Move `query_memgraph_indexed_projects()` out of render path → async model
2. **V-004**: Fix BUNDLE_PATH in upgrade script (`app/target/` not `target/`)
3. **V-005**: Register `helios-terminal.dev` or revert to valid emails

## P1 — Next Sprint

4. **V-002**: Cache all 4 loaders (LazyLock or SingletonEntity)
5. **V-003**: Complete branding sweep (13 remaining strings)
6. **V-006**: Fix YAML parser to use `\n---\n` as end marker
7. **V-007**: Add unit tests for pure parsing functions
8. **V-008**: Default upgrade script to release builds

---

## What Went Right
- ✅ Dynamic agent loading from filesystem — correct approach
- ✅ serde_json for MCP config parsing — robust
- ✅ Fallback patterns when data unavailable
- ✅ Branding consistency across 8+ settings pages
- ✅ Upgrade script structure (numbered steps, error handling, colored output)

## Review Pipeline Stats
- **Raw findings:** 57 across 7 domain agents
- **After dedup/verification:** 11 verified findings
- **Highest-signal domains:** Performance (6 findings → 2 verified), Architecture (6 → 2)
- **Most critical cluster:** Code Indexing page (V-001 render-path subprocess)
