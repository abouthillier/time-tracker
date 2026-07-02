<script lang="ts">
  import { ArrowLeft } from "@lucide/svelte";
  import type {
    ActivitySegment,
    KnownWorkspace,
    SuggestionStateEntry,
    TrackingSettings,
    TrackingStatus,
    WorkspaceMapping,
  } from "../lib/activity";
  import type { ActivitySuggestion } from "../lib/activitySuggestions";
  import type { TimeEntry } from "../lib/timeEntries";
  import type { RmCatalogCache } from "../lib/rm";
  import SuggestionsPanel from "./SuggestionsPanel.svelte";
  import WorkspaceMappings from "./WorkspaceMappings.svelte";

  export let knownWorkspaces: KnownWorkspace[] = [];
  export let workspaceMappings: WorkspaceMapping[] = [];
  export let projectSuggestions: string[] = [];
  export let catalog: RmCatalogCache | null = null;
  export let trackingStatus: TrackingStatus | null = null;
  export let trackingSettings: TrackingSettings;
  export let activitySuggestions: ActivitySuggestion[] = [];
  export let activitySegments: ActivitySegment[] = [];
  export let suggestionState: SuggestionStateEntry[] = [];
  export let entries: TimeEntry[] = [];
  export let selectedDate = "";
  export let onBack: () => void;
  export let onToggleTracking: (enabled: boolean) => void;
  export let onSaveMapping: (
    workspaceKey: string,
    updates: Partial<WorkspaceMapping>,
  ) => void;
  export let onSaveSettings: (settings: TrackingSettings) => void;
  export let onCreateEntry: (suggestion: ActivitySuggestion) => void;
  export let onEditSuggestion: (suggestion: ActivitySuggestion) => void;
  export let onDismissSuggestion: (suggestion: ActivitySuggestion) => void;
</script>

<div class="screen workspaces-screen">
  <header class="screen-header workspaces-header">
    <button
      type="button"
      class="icon-btn"
      aria-label="Back to dashboard"
      on:click={onBack}
    >
      <ArrowLeft />
    </button>

    <h1>Workspaces</h1>

    <label class="tracking-toggle tracking-header-toggle">
      <input
        type="checkbox"
        checked={trackingStatus?.enabled ?? false}
        on:change={(event) =>
          onToggleTracking((event.currentTarget as HTMLInputElement).checked)}
      />
      <span class="tracking-toggle-label">
        <span class="tracking-toggle-status">
          {trackingStatus?.enabled ? "On" : "Off"}
        </span>
        Tracking
      </span>
    </label>
  </header>

  <SuggestionsPanel
    suggestions={activitySuggestions}
    segments={activitySegments}
    mappings={workspaceMappings}
    {suggestionState}
    {entries}
    {selectedDate}
    {onCreateEntry}
    onEdit={onEditSuggestion}
    onDismiss={onDismissSuggestion}
  />
  <WorkspaceMappings
    {knownWorkspaces}
    mappings={workspaceMappings}
    {projectSuggestions}
    {catalog}
    {trackingStatus}
    {trackingSettings}
    {onSaveMapping}
    {onSaveSettings}
  />
</div>
