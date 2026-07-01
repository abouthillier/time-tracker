# Graph Report - time-tracker  (2026-07-01)

## Corpus Check
- 21 files · ~16,925 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 195 nodes · 315 edges · 14 communities
- Extraction: 87% EXTRACTED · 13% INFERRED · 0% AMBIGUOUS · INFERRED: 41 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `fb346df2`
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
- [[_COMMUNITY_Community 9|Community 9]]

## God Nodes (most connected - your core abstractions)
1. `sample_once()` - 11 edges
2. `minutesBetween()` - 9 edges
3. `ActivityTracker` - 9 edges
4. `lock_error()` - 9 edges
5. `resolve_workspace_uri()` - 8 edges
6. `resetForm()` - 7 edges
7. `list_known_workspaces()` - 7 edges
8. `read_active_workspace()` - 6 edges
9. `segments_path()` - 6 edges
10. `read_json_file()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `persist()` --calls--> `saveEntries()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `jumpToToday()` --calls--> `todayKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `moveDay()` --calls--> `shiftDateKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `submitEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `addEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts

## Communities (14 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.07
Nodes (27): ActivitySegment, EditorKind, editorLabel(), KnownWorkspace, startActivityTracking(), SuggestionStateEntry, SuggestionStatus, TrackingSettings (+19 more)

### Community 1 - "Community 1"
Cohesion: 0.11
Nodes (25): ../lib/activity, ../lib/activitySuggestions, ../lib/timeEntries, blockSuggestion, @tauri-apps/api/event, addMinutesToTime(), applyDefaultTimes(), deleteEntry() (+17 more)

### Community 2 - "Community 2"
Cohesion: 0.15
Nodes (17): detect_editor(), ForegroundSample, sample_foreground(), save_suggestion_state(), save_workspace_mappings(), ActivityTracker, app_data_file(), close_stale_open_segment() (+9 more)

### Community 3 - "Community 3"
Cohesion: 0.13
Nodes (16): list_known_workspaces(), list_known_workspaces_command(), load_suggestion_state(), load_tracking_settings(), load_workspace_mappings(), save_tracking_settings(), scan_workspace_storage(), read_json_file() (+8 more)

### Community 4 - "Community 4"
Cohesion: 0.13
Nodes (13): slotsOverlap(), dateToKey(), EntryDraft, entryDuration(), minutesBetween(), saveEntries(), shiftDateKey(), slotDuration() (+5 more)

### Community 5 - "Community 5"
Cohesion: 0.15
Nodes (11): ActivitySegment, default_enabled_editors(), default_poll_interval_secs(), EditorKind, KnownWorkspace, ResolvedWorkspace, SuggestionStateEntry, SuggestionStatus (+3 more)

### Community 6 - "Community 6"
Cohesion: 0.29
Nodes (8): editor_data_dir(), storage_json_path(), workspace_storage_dir(), extract_workspace_uri(), read_active_workspace(), read_json_with_retry(), WorkspaceJsonFile, WorkspaceJsonFolder

### Community 7 - "Community 7"
Cohesion: 0.33
Nodes (8): decode_uri_path(), is_cursor_multi_root_stub(), MultiRootFolder, MultiRootWorkspaceFile, normalize_path_key(), resolve_multi_root_stub(), resolve_remote_workspace(), resolve_workspace_uri()

### Community 8 - "Community 8"
Cohesion: 0.31
Nodes (6): ignoredValue(), labelValue(), mappingFor(), projectValue(), app, @lucide/svelte

### Community 9 - "Community 9"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

## Knowledge Gaps
- **34 isolated node(s):** `[hours, currentMinutes]`, `app`, `EditorKind`, `KnownWorkspace`, `TrackingStatus` (+29 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `minutesBetween()` connect `Community 4` to `Community 0`, `Community 1`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `if()` connect `Community 0` to `Community 1`?**
  _High betweenness centrality (0.036) - this node is a cross-community bridge._
- **Are the 3 inferred relationships involving `sample_once()` (e.g. with `sample_foreground()` and `detect_editor()`) actually correct?**
  _`sample_once()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `minutesBetween()` (e.g. with `submitEntry()` and `addEntry()`) actually correct?**
  _`minutesBetween()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `resolve_workspace_uri()` (e.g. with `read_active_workspace()` and `scan_workspace_storage()`) actually correct?**
  _`resolve_workspace_uri()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `[hours, currentMinutes]`, `app`, `EditorKind` to the rest of the system?**
  _34 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._