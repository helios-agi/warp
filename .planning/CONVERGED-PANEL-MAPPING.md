# Converged Panel → Helios-Terminal Feature Map

## Feature Mapping: helios-desktop converged panel → Warp agent UI

### Legend
- ✅ = Warp already has equivalent
- 🔌 = Needs OSC 777 event bridge (Pi emits, Warp renders)
- 🆕 = Needs new Warp UI component
- ⏭️ = Deferred (complex, lower priority)

### Chat/Message Stream

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| ConvergedMessageStream | Agent View message list | ✅ | Already renders agent/human messages |
| AgentMessageBubble | Agent output blocks | ✅ | Warp renders markdown agent output |
| HumanMessageBubble | User input blocks | ✅ | Warp's terminal input |
| SystemMessageBubble | N/A | 🔌 | New OSC event: `system_message` |
| TypingIndicator | Agent status (thinking) | ✅ | Already shows thinking state |
| ThinkingAccordion/Block | Agent thinking display | ✅ | Warp shows reasoning traces |

### Tool Calls & Code

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| ToolCallCard | CLI agent tool status | 🔌 | Extend `tool_complete` event with rich data: icon, duration, args, result |
| ToolCallChips | Agent status toolbar | ✅ | Warp shows current tool in status bar |
| InlineCodeBlock | Terminal code blocks | ✅ | Native terminal rendering |
| InlineDiffView | Code review diff view | ✅ | `crates/editor` + `code_review/` |
| InlinePatchDiff | Diff visualization | ✅ | Warp has full diff rendering |
| SideBySideDiff | Code review side-by-side | ✅ | `code_review_view.rs` |
| CodeStreamingIndicator | Streaming text | ✅ | Terminal streams natively |

### Composer & Input

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| ConvergedComposer | Rich Input / Agent input | ✅ | Warp's rich agent input with markdown |
| ComposerToolbar | Agent toolbar (model, tokens, cost) | 🔌 | New OSC event: `metrics_update` with token/cost data |
| FileAutocomplete | Slash commands + completions | ✅ | `warp_completer` + `slash_commands/` |
| MentionAutocomplete | N/A | ⏭️ | Not applicable in terminal context |
| SlashCommandAutocomplete | Slash commands | ✅ | Already in `terminal/input/slash_commands/` |
| VoiceInputButton | Voice input | ✅ | `crates/voice_input` |
| ContextChipBar | Context chips | ✅ | Warp has context_chips in input |

### Governance & Approval

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| HitlConfirmModal | Permission request UI | ✅ | `CLIAgentEventType::PermissionRequest` already handled |
| GovernanceStrip | N/A | 🔌 | New OSC event: `governance_status` with score/rules |
| DispatchApprovalDialog | Permission UI | ✅ | Maps to existing permission_request flow |
| DispatchIntentPanel | N/A | 🔌 | New event: `dispatch_intent` with plan preview |

### Mission/Planning

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| MissionBlackboardPanel | N/A | 🔌 | New event: `blackboard_update` with intent/plan/contributions |
| MissionMiniView | N/A | 🔌 | New event: `mission_status` with phase/progress |
| PlanCompilerCard | N/A | 🔌 | New event: `plan_update` with tasks/dependencies |
| PhaseMarkerCard | Agent progress steps | ✅ | `progress.rs` already renders multi-step flows |
| PipelineStageCard | N/A | ⏭️ | Complex pipeline visualization |

### Subagent/Coordination

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| SubagentProgressCard | Child agent status | 🔌 | New event: `subagent_update` with id/role/status/progress |
| CoordinationStrip | N/A | 🔌 | New event: `coordination_status` with active agents |
| MeshMiniView | N/A | 🔌 | New event: `mesh_state` with connected agents |
| MeshStateStrip | N/A | 🔌 | Same as mesh_state event |
| PresenceDots | N/A | ⏭️ | Multi-user presence not in scope |

### Context & Intelligence

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| BrainStrip | N/A | 🔌 | New event: `brain_status` with retrieval channels |
| CortexStrip | N/A | 🔌 | New event: `cortex_status` with active policy |
| HemaContextPane | N/A | 🔌 | New event: `hema_context` with dispatch plan |
| EvidenceContextCard | N/A | 🔌 | New event: `evidence_pack` with citations |
| EvidencePackCard | N/A | 🔌 | Same as evidence_pack |

### Navigation & Chrome

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| ConvergedHeader | Agent view header | ✅ | Warp has agent view header with model picker |
| ScopeBreadcrumb | N/A | ⏭️ | Terminal doesn't need breadcrumbs |
| SessionHistoryPanel | Conversation history | ✅ | `agent_conversations_model.rs` |
| ProjectQuickPicker | N/A | ✅ | Terminal has cwd/project awareness |
| AgentHealthBadge | Agent status indicator | ✅ | Warp shows agent status in toolbar |
| MaintenanceBadge | N/A | 🔌 | New event: `maintenance_status` |
| CheckpointDot | N/A | 🔌 | New event: `checkpoint` with label |

### Decision/Review

| helios-desktop Component | Warp Equivalent | Status | Bridge Strategy |
|--------------------------|-----------------|--------|-----------------|
| DecisionCard | N/A | 🔌 | New event: `decision_request` with options |
| BlackboardDecisionCard | N/A | 🔌 | Same as decision_request |
| DependencyChangeCard | N/A | ⏭️ | Complex dependency visualization |
| InterviewFormInline | Question UI | 🔌 | New event: `interview_request` with questions |
| InterviewLink | Link to interview | ✅ | Terminal can render clickable links |

### Chat Adapters

| helios-desktop Adapter | Warp Strategy | Status |
|------------------------|---------------|--------|
| BrainChatAdapter | System messages via OSC | 🔌 |
| GovernanceChatAdapter | System messages via OSC | 🔌 |
| MeshChatAdapter | System messages via OSC | 🔌 |
| MissionChatAdapter | System messages via OSC | 🔌 |

### Summary

| Category | Total | ✅ Already | 🔌 Bridge | 🆕 New | ⏭️ Deferred |
|----------|-------|-----------|-----------|--------|-------------|
| Chat/Messages | 6 | 5 | 1 | 0 | 0 |
| Tool Calls | 7 | 6 | 1 | 0 | 0 |
| Composer | 7 | 6 | 1 | 0 | 0 |
| Governance | 4 | 2 | 2 | 0 | 0 |
| Mission/Planning | 5 | 1 | 3 | 0 | 1 |
| Subagent/Coord | 5 | 0 | 3 | 0 | 2 |
| Context/Intel | 5 | 0 | 5 | 0 | 0 |
| Navigation | 7 | 5 | 2 | 0 | 0 |
| Decision/Review | 5 | 1 | 3 | 0 | 1 |
| Chat Adapters | 4 | 0 | 4 | 0 | 0 |
| **TOTAL** | **55** | **26** | **25** | **0** | **4** |

47% already covered by Warp. 45% need OSC 777 event bridge extensions. 7% deferred.

## Implementation: Extended OSC 777 Protocol

All 25 bridge features use the SAME mechanism: extend the OSC 777 event protocol
with new event types. Pi emits them, Warp's listener renders them.

New event types needed in `helios-terminal-bridge.ts`:
1. `system_message` — governance, brain, mesh, mission system messages
2. `metrics_update` — token counts, cost, model info
3. `governance_status` — compliance score, active rules
4. `subagent_update` — subagent id, role, status, progress, files
5. `mission_status` — phase, plan state, blackboard data
6. `decision_request` — options for user to choose
7. `interview_request` — structured form for user input
8. `checkpoint` — named checkpoint in agent workflow
9. `evidence_pack` — citations and evidence
10. `coordination_status` — active agents, file reservations
