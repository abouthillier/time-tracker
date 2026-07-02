<script lang="ts">
  import {
    applyLegacyResolutions,
    categoriesForCatalog,
    findCatalogProject,
    suggestRmProjects,
    type LegacyProjectResolution,
  } from "../lib/rmCatalog";
  import type { WorkspaceMapping } from "../lib/activity";
  import type { TimeEntry } from "../lib/timeEntries";
  import type { RmCatalogCache } from "../lib/rm";

  export let open = false;
  export let catalog: RmCatalogCache;
  export let entries: TimeEntry[] = [];
  export let mappings: WorkspaceMapping[] = [];
  export let legacyKeys: string[] = [];
  export let onComplete: (result: {
    entries: TimeEntry[];
    mappings: WorkspaceMapping[];
  }) => void;
  export let onClose: () => void;

  let currentIndex = 0;
  let selectedAssignableId = "";
  let selectedCategory = "";
  let error = "";
  let resolutions: Record<string, LegacyProjectResolution> = {};
  let wasOpen = false;

  $: currentKey = legacyKeys[currentIndex] ?? "";
  $: categoryOptions = categoriesForCatalog(catalog);
  $: suggestedProjectId = currentKey
    ? String(suggestRmProjects(catalog, currentKey, 1)[0]?.assignableId ?? "")
    : "";

  $: if (open && !wasOpen) {
    currentIndex = 0;
    resolutions = {};
    wasOpen = true;
  } else if (!open) {
    wasOpen = false;
  }

  $: if (open && currentKey) {
    selectedAssignableId = suggestedProjectId;
    selectedCategory = categoryOptions[0] ?? "";
    error = "";
  }

  const handleProjectChange = () => {
    error = "";
  };

  const skipKey = () => {
    if (currentIndex + 1 >= legacyKeys.length) {
      finishMigration();
      return;
    }

    currentIndex += 1;
  };

  const saveStep = () => {
    error = "";

    if (!selectedAssignableId) {
      error = "Choose a Resource Management project.";
      return;
    }

    const project = findCatalogProject(catalog, Number(selectedAssignableId));
    if (!project) {
      error = "Choose a valid Resource Management project.";
      return;
    }

    if (!selectedCategory.trim() && categoryOptions.length > 0) {
      error = "Choose a category.";
      return;
    }

    resolutions = {
      ...resolutions,
      [currentKey]: {
        assignableId: project.assignableId,
        project: project.name,
        category: selectedCategory.trim(),
      },
    };

    if (currentIndex + 1 >= legacyKeys.length) {
      finishMigration();
      return;
    }

    currentIndex += 1;
  };

  const finishMigration = () => {
    const result = applyLegacyResolutions({
      entries,
      mappings,
      resolutions,
    });

    onComplete(result);
    onClose();
  };
</script>

{#if open && legacyKeys.length > 0}
  <dialog class="migration-modal" open aria-modal="true">
    <div class="migration-card glass-card">
      <p class="eyebrow">One-time setup</p>
      <h2>Map local projects to Resource Management</h2>
      <p class="muted-copy">
        Step {currentIndex + 1} of {legacyKeys.length}: match
        <strong>{currentKey}</strong>
        to an RM project and category.
      </p>

      <label>
        <span>Resource Management project</span>
        <select bind:value={selectedAssignableId} on:change={handleProjectChange}>
          <option value="">Select a project</option>
          {#each catalog.projects as project (project.assignableId)}
            <option value={String(project.assignableId)}>
              {project.name}
            </option>
          {/each}
        </select>
      </label>

      {#if categoryOptions.length > 0}
        <label>
          <span>Category</span>
          <select bind:value={selectedCategory}>
            <option value="">Select a category</option>
            {#each categoryOptions as categoryName}
              <option value={categoryName}>{categoryName}</option>
            {/each}
          </select>
        </label>
      {:else}
        <p class="muted-copy">
          No account categories were returned from RM. Refresh the catalog or
          enter categories in Resource Management.
        </p>
      {/if}

      {#if error}
        <p class="form-error" role="alert">{error}</p>
      {/if}

      <div class="migration-actions">
        <button type="button" class="ghost" on:click={onClose}>Later</button>
        <button type="button" class="ghost" on:click={skipKey}>Skip</button>
        <button type="button" class="primary" on:click={saveStep}>
          {currentIndex + 1 >= legacyKeys.length ? "Finish" : "Next"}
        </button>
      </div>
    </div>
  </dialog>
{/if}

<style>
  .migration-modal {
    position: fixed;
    inset: 0;
    margin: auto;
    border: none;
    padding: 0;
    background: transparent;
    width: min(480px, calc(100% - 32px));
    max-height: calc(100dvh - 32px);
    overflow: auto;
  }

  .migration-modal::backdrop {
    background: rgba(0, 0, 0, 0.55);
  }

  .migration-card {
    border: 1px solid var(--bg-card-border);
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;

    p {
      color: var(--text-muted);
    }
  }

  .migration-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
