<script lang="ts">
  import type { ActivitySuggestion } from "../lib/activitySuggestions";

  export let project = "";
  export let startTime = "";
  export let endTime = "";
  export let notes = "";
  export let error = "";
  export let isSaving = false;
  export let isLoading = false;
  export let isEditing = false;
  export let pendingSuggestion: ActivitySuggestion | null = null;
  export let projectSuggestions: string[] = [];
  export let onSubmit: () => void;
  export let onCancel: () => void;

  $: isCreatingFromSuggestion = pendingSuggestion !== null;
</script>

<form
  class:editing={isEditing}
  class:from-suggestion={isCreatingFromSuggestion}
  class="entry-form"
  on:submit|preventDefault={onSubmit}
>
  {#if isEditing}
    <div class="editing-banner">
      <span>Editing time slot</span>
      <button type="button" class="ghost" on:click={onCancel}>Cancel</button>
    </div>
  {:else if pendingSuggestion}
    <div class="editing-banner">
      <span>
        Creating entry for {pendingSuggestion.workspaceLabel}{#if !pendingSuggestion.project}
          — pick a project below{/if}
      </span>
      <button type="button" class="ghost" on:click={onCancel}>Cancel</button>
    </div>
  {/if}

  <label>
    <span>Project</span>
    <input
      bind:value={project}
      list="entry-form-project-suggestions"
      name="project"
      placeholder="Client work, admin, research"
    />
    <datalist id="entry-form-project-suggestions">
      {#each projectSuggestions as suggestedProject}
        <option value={suggestedProject}></option>
      {/each}
    </datalist>
  </label>

  <div class="time-grid">
    <label>
      <span>Start</span>
      <input bind:value={startTime} name="startTime" type="time" />
    </label>

    <label>
      <span>End</span>
      <input bind:value={endTime} name="endTime" type="time" />
    </label>
  </div>

  <label>
    <span>Notes <small>optional</small></span>
    <textarea
      bind:value={notes}
      name="notes"
      rows="3"
      placeholder="What did you work on?"
    ></textarea>
  </label>

  {#if error}
    <p class="form-error" role="alert">{error}</p>
  {/if}

  <button class="primary" type="submit" disabled={isSaving || isLoading}>
    {isSaving
      ? "Saving..."
      : isEditing
        ? "Save changes"
        : isCreatingFromSuggestion
          ? "Create entry"
          : "Add entry"}
  </button>
</form>
