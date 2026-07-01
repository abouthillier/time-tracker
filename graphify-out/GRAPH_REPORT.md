# Graph Report - time-tracker  (2026-07-01)

## Corpus Check
- 25 files · ~17,788 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 219 nodes · 380 edges · 13 communities
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 53 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2bcad32e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]

## God Nodes (most connected - your core abstractions)
1. `sample_once()` - 11 edges
2. `./WorkspaceMappings.svelte` - 10 edges
3. `resetForm()` - 9 edges
4. `minutesBetween()` - 9 edges
5. `ActivityTracker` - 9 edges
6. `lock_error()` - 9 edges
7. `refreshActivityData()` - 8 edges
8. `submitEntry()` - 8 edges
9. `./SuggestionsPanel.svelte` - 8 edges
10. `resolve_workspace_uri()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `persist()` --calls--> `saveEntries()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `refreshActivityData()` --calls--> `loadActivitySegments()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadWorkspaceMappings()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadSuggestionState()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `listKnownWorkspaces()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts

## Communities (13 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.08
Nodes (41): ../lib/activity, ../lib/activitySuggestions, ../lib/timeEntries, editing, blockSuggestion, @tauri-apps/api/event, saveSuggestionState(), addMinutesToTime() (+33 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (39): ActivitySegment, EditorKind, editorLabel(), getTrackingStatus(), KnownWorkspace, listKnownWorkspaces(), loadActivitySegments(), loadSuggestionState() (+31 more)

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (25): detect_editor(), ForegroundSample, sample_foreground(), list_known_workspaces(), list_known_workspaces_command(), load_suggestion_state(), load_tracking_settings(), load_workspace_mappings() (+17 more)

### Community 3 - "Community 3"
Cohesion: 0.13
Nodes (17): slotsOverlap(), dateFromKey(), dateToKey(), EntryDraft, entryDuration(), formatDateLabel(), formatDateSubLabel(), formatWeekdayLabel() (+9 more)

### Community 4 - "Community 4"
Cohesion: 0.15
Nodes (17): editor_data_dir(), storage_json_path(), workspace_storage_dir(), extract_workspace_uri(), read_active_workspace(), read_json_with_retry(), scan_workspace_storage(), WorkspaceJsonFile (+9 more)

### Community 5 - "Community 5"
Cohesion: 0.15
Nodes (11): ActivitySegment, default_enabled_editors(), default_poll_interval_secs(), EditorKind, KnownWorkspace, ResolvedWorkspace, SuggestionStateEntry, SuggestionStatus (+3 more)

### Community 6 - "Community 6"
Cohesion: 0.28
Nodes (7): entries_path(), load_entries(), run(), save_entries(), TimeEntry, TimeSlot, main()

### Community 7 - "Community 7"
Cohesion: 0.43
Nodes (6): ignored, ignoredValue(), labelValue(), mappingFor(), projectValue(), ./WorkspaceMappings.svelte

### Community 8 - "Community 8"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

## Knowledge Gaps
- **36 isolated node(s):** `hasProject`, `existing`, `workspace`, `WorkspaceMapping`, `app` (+31 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `minutesBetween()` connect `Community 3` to `Community 0`, `Community 1`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **Why does `submitEntry()` connect `Community 0` to `Community 3`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Why does `./WorkspaceMappings.svelte` connect `Community 7` to `Community 0`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **Are the 3 inferred relationships involving `sample_once()` (e.g. with `sample_foreground()` and `detect_editor()`) actually correct?**
  _`sample_once()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `minutesBetween()` (e.g. with `submitEntry()` and `addEntry()`) actually correct?**
  _`minutesBetween()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `hasProject`, `existing`, `workspace` to the rest of the system?**
  _36 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._