# Graph Report - time-tracker  (2026-06-30)

## Corpus Check
- 10 files · ~11,795 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 63 nodes · 88 edges · 12 communities (11 shown, 1 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 6 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `40c242e3`
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

## God Nodes (most connected - your core abstractions)
1. `resetForm()` - 7 edges
2. `submitEntry()` - 6 edges
3. `applyDefaultTimes()` - 6 edges
4. `minutesBetween()` - 6 edges
5. `deleteSlot()` - 5 edges
6. `persist()` - 5 edges
7. `moveEditedSlot()` - 5 edges
8. `Svelte + TS + Vite` - 4 edges
9. `deleteEntry()` - 4 edges
10. `moveDay()` - 3 edges

## Surprising Connections (you probably didn't know these)
- `submitEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `persist()` --calls--> `saveEntries()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `addEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `moveDay()` --calls--> `shiftDateKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `jumpToToday()` --calls--> `todayKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts

## Communities (12 total, 1 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.16
Nodes (8): EntryDraft, entryDuration(), minutesBetween(), slotDuration(), TimeEntry, TimeSlot, timeToMinutes(), addEntry()

### Community 1 - "Community 1"
Cohesion: 0.28
Nodes (7): entries_path(), load_entries(), run(), save_entries(), TimeEntry, TimeSlot, main()

### Community 2 - "Community 2"
Cohesion: 0.29
Nodes (4): ./lib/timeEntries, hasProject, isSameProject(), normalizeProject()

### Community 3 - "Community 3"
Cohesion: 0.38
Nodes (7): saveEntries(), deleteSlot(), moveEditedSlot(), persist(), removeSlot(), submitEntry(), upsertSlot()

### Community 4 - "Community 4"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

### Community 5 - "Community 5"
Cohesion: 0.5
Nodes (5): addMinutesToTime(), applyDefaultTimes(), deleteEntry(), getDefaultStartTime(), resetForm()

### Community 6 - "Community 6"
Cohesion: 0.4
Nodes (5): dateToKey(), shiftDateKey(), todayKey(), jumpToToday(), moveDay()

## Knowledge Gaps
- **10 isolated node(s):** `hasProject`, `app`, `TimeSlot`, `TimeEntry`, `EntryDraft` (+5 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `minutesBetween()` connect `Community 0` to `Community 3`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Why does `submitEntry()` connect `Community 3` to `Community 0`, `Community 2`, `Community 5`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **Why does `persist()` connect `Community 3` to `Community 2`, `Community 5`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `minutesBetween()` (e.g. with `submitEntry()` and `addEntry()`) actually correct?**
  _`minutesBetween()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `hasProject`, `app`, `TimeSlot` to the rest of the system?**
  _10 weakly-connected nodes found - possible documentation gaps or missing edges._