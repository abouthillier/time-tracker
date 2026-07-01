<script lang="ts">
  import {
    editorLabel,
    type ActivitySegment,
    type SuggestionStateEntry,
    type WorkspaceMapping,
  } from "../lib/activity";
  import {
    buildWorkspaceActivityGroups,
    type ActivitySuggestion,
    type TrackedActivityBlock,
  } from "../lib/activitySuggestions";
  import { formatMinutes, type TimeEntry } from "../lib/timeEntries";

  export let suggestions: ActivitySuggestion[] = [];
  export let segments: ActivitySegment[] = [];
  export let mappings: WorkspaceMapping[] = [];
  export let selectedDate = "";
  export let suggestionState: SuggestionStateEntry[] = [];
  export let entries: TimeEntry[] = [];
  export let onCreateEntry: (suggestion: ActivitySuggestion) => void;
  export let onEdit: (suggestion: ActivitySuggestion) => void;
  export let onDismiss: (suggestion: ActivitySuggestion) => void;

  $: workspaceGroups = buildWorkspaceActivityGroups({
    segments,
    mappings,
    suggestionState,
    entries,
    dateKey: selectedDate,
  });

  const blockDurationLabel = (block: TrackedActivityBlock) => {
    if (block.isActive) {
      return `${formatMinutes(block.durationMinutes)} (active)`;
    }

    return formatMinutes(block.durationMinutes);
  };

  const suggestionForBlock = (block: TrackedActivityBlock) =>
    suggestions.find((suggestion) => suggestion.id === block.id);
</script>

<section class="suggestions-card" aria-labelledby="suggestions-heading">
  <div class="entries-header">
    <div>
      <p class="eyebrow">Suggestions</p>
      <h2 id="suggestions-heading">Workspace activity</h2>
    </div>
  </div>

  {#if workspaceGroups.length === 0}
    <p class="empty-state">
      No workspace activity logged for this day yet. Work in Cursor or VS Code
      with tracking enabled, then check back here.
    </p>
  {:else}
    <ul class="workspace-group-list">
      {#each workspaceGroups as group (group.workspaceKey)}
        <li class="workspace-group">
          <div class="workspace-group-header">
            <div>
              <div class="entry-title">
                <strong>{group.workspaceLabel}</strong>
                <span>{formatMinutes(group.totalMinutes)}</span>
              </div>
              <p class="workspace-meta">
                {editorLabel(group.editor)}
                {#if group.project}
                  · {group.project}
                {:else}
                  · <em>Unmapped</em>
                {/if}
              </p>
            </div>

            {#if group.consolidatedSuggestion}
              <button
                type="button"
                class="primary"
                on:click={() => onCreateEntry(group.consolidatedSuggestion!)}
              >
                Create entry
              </button>
            {/if}
          </div>

          <ul class="tracked-activity-list">
            {#each group.blocks as block (block.id)}
              {@const blockSuggestion = suggestionForBlock(block)}
              <li class:active={block.isActive}>
                <div class="tracked-block-row">
                  <div>
                    <p class="tracked-block-time">
                      {block.startTime}
                      {#if block.endTime}
                        – {block.endTime}
                      {:else}
                        – now
                      {/if}
                      <span>{blockDurationLabel(block)}</span>
                    </p>

                    {#if blockSuggestion?.overlapsExisting}
                      <p class="suggestion-warning" role="status">
                        Overlaps an existing entry.
                      </p>
                    {/if}
                  </div>

                  {#if blockSuggestion}
                    <div class="suggestion-actions">
                      <button
                        type="button"
                        class="primary"
                        on:click={() => onCreateEntry(blockSuggestion)}
                      >
                        Create entry
                      </button>
                      <button type="button" on:click={() => onEdit(blockSuggestion)}>
                        Edit
                      </button>
                      <button
                        type="button"
                        class="ghost"
                        on:click={() => onDismiss(blockSuggestion)}
                      >
                        Dismiss
                      </button>
                    </div>
                  {/if}
                </div>
              </li>
            {/each}
          </ul>
        </li>
      {/each}
    </ul>
  {/if}
</section>
