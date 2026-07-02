# Graph Report - time-tracker  (2026-07-02)

## Corpus Check
- 32 files · ~20,307 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 290 nodes · 496 edges · 17 communities
- Extraction: 84% EXTRACTED · 16% INFERRED · 0% AMBIGUOUS · INFERRED: 77 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `943306ca`
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
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]

## God Nodes (most connected - your core abstractions)
1. `sample_once()` - 11 edges
2. `refresh_catalog()` - 11 edges
3. `./WorkspaceMappings.svelte` - 10 edges
4. `resetForm()` - 9 edges
5. `minutesBetween()` - 9 edges
6. `ActivityTracker` - 9 edges
7. `lock_error()` - 9 edges
8. `refreshActivityData()` - 8 edges
9. `submitEntry()` - 8 edges
10. `./SuggestionsPanel.svelte` - 8 edges

## Surprising Connections (you probably didn't know these)
- `rm_load_settings()` --calls--> `load_settings()`  [INFERRED]
  src-tauri/src/rm/mod.rs → src-tauri/src/rm/settings.rs
- `refreshActivityData()` --calls--> `loadActivitySegments()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadWorkspaceMappings()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadSuggestionState()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `listKnownWorkspaces()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts

## Communities (17 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.07
Nodes (41): ../lib/activity, ../lib/activitySuggestions, ../lib/timeEntries, editing, blockSuggestion, saveSuggestionState(), saveEntries(), addMinutesToTime() (+33 more)

### Community 1 - "Community 1"
Cohesion: 0.08
Nodes (36): detect_editor(), ForegroundSample, sample_foreground(), list_known_workspaces(), list_known_workspaces_command(), load_suggestion_state(), load_tracking_settings(), load_workspace_mappings() (+28 more)

### Community 2 - "Community 2"
Cohesion: 0.09
Nodes (33): editorLabel(), ActivitySuggestion, buildActivitySuggestions(), buildConsolidatedSuggestion(), buildTrackedActivity(), buildWorkspaceActivityGroups(), createSuggestionId(), formatSuggestionSummary() (+25 more)

### Community 3 - "Community 3"
Cohesion: 0.13
Nodes (21): ../lib/rm, clearFeedback(), loadState(), refreshCatalog(), removeToken(), saveToken(), testConnection(), tokenForActions() (+13 more)

### Community 4 - "Community 4"
Cohesion: 0.11
Nodes (22): ActivitySegment, EditorKind, getTrackingStatus(), KnownWorkspace, listKnownWorkspaces(), loadActivitySegments(), loadSuggestionState(), loadTrackingSettings() (+14 more)

### Community 5 - "Community 5"
Cohesion: 0.15
Nodes (16): editor_data_dir(), storage_json_path(), workspace_storage_dir(), extract_workspace_uri(), read_active_workspace(), read_json_with_retry(), WorkspaceJsonFile, WorkspaceJsonFolder (+8 more)

### Community 6 - "Community 6"
Cohesion: 0.21
Nodes (12): emit_progress(), fetch_all_projects(), fetch_project_categories(), non_empty(), project_client_label(), refresh_catalog(), test_connection(), require_token() (+4 more)

### Community 7 - "Community 7"
Cohesion: 0.15
Nodes (11): ActivitySegment, default_enabled_editors(), default_poll_interval_secs(), EditorKind, KnownWorkspace, ResolvedWorkspace, SuggestionStateEntry, SuggestionStatus (+3 more)

### Community 8 - "Community 8"
Cohesion: 0.2
Nodes (8): RmBudgetCategory, RmCatalogCache, RmCatalogProgress, RmPagedResponse, RmPaging, RmProject, RmProjectCatalogEntry, RmSettings

### Community 9 - "Community 9"
Cohesion: 0.36
Nodes (3): format_http_error(), rate_limit_wait(), RmClient

### Community 10 - "Community 10"
Cohesion: 0.43
Nodes (6): ignored, ignoredValue(), labelValue(), mappingFor(), projectValue(), ./WorkspaceMappings.svelte

### Community 11 - "Community 11"
Cohesion: 0.47
Nodes (5): entries_path(), load_entries(), save_entries(), TimeEntry, TimeSlot

### Community 12 - "Community 12"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

## Knowledge Gaps
- **47 isolated node(s):** `hasProject`, `existing`, `workspace`, `WorkspaceMapping`, `app` (+42 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `minutesBetween()` connect `Community 2` to `Community 0`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `refresh_catalog()` connect `Community 6` to `Community 9`, `Community 1`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Are the 3 inferred relationships involving `sample_once()` (e.g. with `sample_foreground()` and `detect_editor()`) actually correct?**
  _`sample_once()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `refresh_catalog()` (e.g. with `.category_fetch_delay()` and `save_catalog()`) actually correct?**
  _`refresh_catalog()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `minutesBetween()` (e.g. with `submitEntry()` and `addEntry()`) actually correct?**
  _`minutesBetween()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `hasProject`, `existing`, `workspace` to the rest of the system?**
  _47 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._