<script lang="ts">
  import type { ActivitySuggestion } from "../lib/activitySuggestions";
  import {
    categoriesForCatalog,
    findCatalogProject,
    isCatalogReady,
  } from "../lib/rmCatalog";
  import type { RmCatalogCache } from "../lib/rm";

  export let project = "";
  export let assignableId: number | null = null;
  export let category = "";
  export let startTime = "";
  export let endTime = "";
  export let notes = "";
  export let error = "";
  export let isSaving = false;
  export let isLoading = false;
  export let isEditing = false;
  export let pendingSuggestion: ActivitySuggestion | null = null;
  export let catalog: RmCatalogCache | null = null;
  export let onSubmit: () => void;
  export let onCancel: () => void;

  $: useRmPickers = isCatalogReady(catalog);
  $: categoryOptions = categoriesForCatalog(catalog);
  $: isCreatingFromSuggestion = pendingSuggestion !== null;

  const handleProjectChange = (event: Event) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    assignableId = value ? Number(value) : null;
    const nextProject = findCatalogProject(catalog, assignableId);
    project = nextProject?.name ?? "";
  };
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

  {#if useRmPickers}
    <label>
      <span>Project</span>
      <select
        value={assignableId != null ? String(assignableId) : ""}
        on:change={handleProjectChange}
        required
      >
        <option value="">Select a project</option>
        {#each catalog?.projects ?? [] as rmProject (rmProject.assignableId)}
          <option value={String(rmProject.assignableId)}>
            {rmProject.name}
          </option>
        {/each}
      </select>
    </label>

    <label>
      <span>Category</span>
      {#if categoryOptions.length > 0}
        <select bind:value={category} required>
          <option value="">Select a category</option>
          {#each categoryOptions as categoryName}
            <option value={categoryName}>{categoryName}</option>
          {/each}
        </select>
      {:else}
        <input
          bind:value={category}
          placeholder="No RM categories loaded"
        />
      {/if}
    </label>
  {:else}
    <label>
      <span>Project</span>
      <input
        bind:value={project}
        name="project"
        placeholder="Client work, admin, research"
        required
      />
    </label>
  {/if}

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
