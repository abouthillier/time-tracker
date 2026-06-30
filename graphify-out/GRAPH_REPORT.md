# Graph Report - time-tracker  (2026-06-30)

## Corpus Check
- 10 files · ~11,337 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 49 nodes · 63 edges · 13 communities (11 shown, 2 thin omitted)
- Extraction: 90% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 6 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

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
1. `resetForm()` - 6 edges
2. `applyDefaultTimes()` - 5 edges
3. `minutesBetween()` - 5 edges
4. `submitEntry()` - 4 edges
5. `deleteEntry()` - 4 edges
6. `persist()` - 4 edges
7. `Svelte + TS + Vite` - 4 edges
8. `moveDay()` - 3 edges
9. `jumpToToday()` - 3 edges
10. `todayKey()` - 3 edges

## Surprising Connections (you probably didn't know these)
- `submitEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `moveDay()` --calls--> `shiftDateKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `jumpToToday()` --calls--> `todayKey()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `addEntry()` --calls--> `minutesBetween()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts
- `persist()` --calls--> `saveEntries()`  [INFERRED]
  src/App.svelte → src/lib/timeEntries.ts

## Communities (13 total, 2 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.32
Nodes (6): entries_path(), load_entries(), run(), save_entries(), TimeEntry, main()

### Community 2 - "Community 2"
Cohesion: 0.47
Nodes (4): ./lib/timeEntries, addMinutesToTime(), applyDefaultTimes(), getDefaultStartTime()

### Community 3 - "Community 3"
Cohesion: 0.33
Nodes (5): code:ts (// store.ts), Need an official Svelte framework?, Recommended IDE Setup, Svelte + TS + Vite, Technical considerations

### Community 4 - "Community 4"
Cohesion: 0.5
Nodes (4): entryDuration(), minutesBetween(), timeToMinutes(), addEntry()

### Community 5 - "Community 5"
Cohesion: 0.5
Nodes (4): saveEntries(), deleteEntry(), persist(), submitEntry()

### Community 6 - "Community 6"
Cohesion: 0.67
Nodes (3): dateToKey(), shiftDateKey(), todayKey()

### Community 7 - "Community 7"
Cohesion: 0.67
Nodes (3): jumpToToday(), moveDay(), resetForm()

## Knowledge Gaps
- **7 isolated node(s):** `app`, `TimeEntry`, `EntryDraft`, `TimeEntry`, `Recommended IDE Setup` (+2 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `minutesBetween()` connect `Community 4` to `Community 1`, `Community 5`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **Why does `submitEntry()` connect `Community 5` to `Community 2`, `Community 4`, `Community 7`?**
  _High betweenness centrality (0.033) - this node is a cross-community bridge._
- **Why does `persist()` connect `Community 5` to `Community 2`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `minutesBetween()` (e.g. with `submitEntry()` and `addEntry()`) actually correct?**
  _`minutesBetween()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `app`, `TimeEntry`, `EntryDraft` to the rest of the system?**
  _7 weakly-connected nodes found - possible documentation gaps or missing edges._