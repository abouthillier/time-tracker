# Graph Report - time-tracker  (2026-07-16)

## Corpus Check
- 41 files · ~28,988 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 461 nodes · 816 edges · 21 communities
- Extraction: 84% EXTRACTED · 16% INFERRED · 0% AMBIGUOUS · INFERRED: 131 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `118f2925`
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
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]

## God Nodes (most connected - your core abstractions)
1. `./WorkspaceMappings.svelte` - 16 edges
2. `refresh_catalog()` - 13 edges
3. `sample_once()` - 11 edges
4. `refreshRmData()` - 10 edges
5. `submitEntry()` - 10 edges
6. `invokeErrorMessage()` - 10 edges
7. `sync_day()` - 10 edges
8. `resetForm()` - 9 edges
9. `minutesBetween()` - 9 edges
10. `ActivityTracker` - 9 edges

## Surprising Connections (you probably didn't know these)
- `sync_day()` --calls--> `verify_linked_user()`  [INFERRED]
  src-tauri/src/rm/sync.rs → src-tauri/src/rm/identity.rs
- `refreshActivityData()` --calls--> `loadActivitySegments()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadWorkspaceMappings()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `loadSuggestionState()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts
- `refreshActivityData()` --calls--> `listKnownWorkspaces()`  [INFERRED]
  src/App.svelte → src/lib/activity.ts

## Communities (21 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.07
Nodes (44): saveSuggestionState(), moveEditedSlot(), prepareEntriesForSave(), removeSlot(), saveEntries(), todayKey(), updateEntryById(), upsertSlot() (+36 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (45): clearFeedback(), confirmLink(), ensureTokenReady(), findIdentityUser(), loadState(), refreshCatalog(), removeLinkedUser(), removeToken() (+37 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (33): detect_editor(), ForegroundSample, sample_foreground(), list_known_workspaces(), list_known_workspaces_command(), load_suggestion_state(), load_tracking_settings(), load_workspace_mappings() (+25 more)

### Community 3 - "Community 3"
Cohesion: 0.07
Nodes (39): cleared, edited, errored, freshDate, linked, resolved, staleDate, synced (+31 more)

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (43): ActivitySegment, EditorKind, editorLabel(), getTrackingStatus(), KnownWorkspace, listKnownWorkspaces(), loadActivitySegments(), loadSuggestionState() (+35 more)

### Community 5 - "Community 5"
Cohesion: 0.08
Nodes (29): ../lib/activity, ../lib/activitySuggestions, ../lib/rm, ../lib/rmCatalog, ../lib/timeEntries, dirty, error, synced (+21 more)

### Community 6 - "Community 6"
Cohesion: 0.1
Nodes (28): write_json_file(), test_connection(), fetch_all_users(), find_user_candidates(), get_linked_user(), link_user(), load_settings_linked(), require_token() (+20 more)

### Community 7 - "Community 7"
Cohesion: 0.09
Nodes (20): RmBudgetCategory, RmBudgetItem, RmCatalogCache, RmCatalogProgress, RmCreatedTimeEntry, RmLinkedUser, RmPagedResponse, RmPaging (+12 more)

### Community 8 - "Community 8"
Cohesion: 0.15
Nodes (16): editor_data_dir(), storage_json_path(), workspace_storage_dir(), extract_workspace_uri(), read_active_workspace(), read_json_with_retry(), WorkspaceJsonFile, WorkspaceJsonFolder (+8 more)

### Community 9 - "Community 9"
Cohesion: 0.15
Nodes (16): RmCatalogCache, RmProjectCatalogEntry, acronymFor(), catalogAgeDays(), categoriesForCatalog(), categoriesForProject(), collectLegacyProjectKeys(), findCatalogProjectByName() (+8 more)

### Community 10 - "Community 10"
Cohesion: 0.21
Nodes (18): rm_sync_day(), build_payload(), build_payload_aggregates_hours_and_hash(), EntryPayload, post_entry(), push_entry(), put_entry(), put_update_body() (+10 more)

### Community 11 - "Community 11"
Cohesion: 0.15
Nodes (11): ActivitySegment, default_enabled_editors(), default_poll_interval_secs(), EditorKind, KnownWorkspace, ResolvedWorkspace, SuggestionStateEntry, SuggestionStatus (+3 more)

### Community 12 - "Community 12"
Cohesion: 0.28
Nodes (14): build_probe_order(), discover_timesheet_categories(), emit_progress(), fetch_account_categories(), fetch_all_projects(), fetch_paginated_category_names(), fetch_project_budget_categories(), fetch_project_categories() (+6 more)

### Community 13 - "Community 13"
Cohesion: 0.36
Nodes (3): format_http_error(), rate_limit_wait(), RmClient

### Community 14 - "Community 14"
Cohesion: 0.38
Nodes (6): entries_path(), load_entries(), RmEntrySync, save_entries(), TimeEntry, TimeSlot

### Community 15 - "Community 15"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

## Knowledge Gaps
- **92 isolated node(s):** `hasProject`, `trimmedCategory`, `catalogProject`, `mapping`, `app` (+87 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `read_json_file()` connect `Community 2` to `Community 6`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Why does `refresh_catalog()` connect `Community 12` to `Community 13`, `Community 6`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **Why does `load_settings()` connect `Community 6` to `Community 2`, `Community 12`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `refresh_catalog()` (e.g. with `save_catalog()` and `load_settings()`) actually correct?**
  _`refresh_catalog()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `sample_once()` (e.g. with `sample_foreground()` and `detect_editor()`) actually correct?**
  _`sample_once()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `refreshRmData()` (e.g. with `loadRmCatalog()` and `loadRmSettings()`) actually correct?**
  _`refreshRmData()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `submitEntry()` (e.g. with `minutesBetween()` and `upsertSlot()`) actually correct?**
  _`submitEntry()` has 3 INFERRED edges - model-reasoned connections that need verification._