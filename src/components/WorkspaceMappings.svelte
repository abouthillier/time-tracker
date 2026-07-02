<script lang="ts">
  import {
    editorLabel,
    type EditorKind,
    type KnownWorkspace,
    type TrackingSettings,
    type TrackingStatus,
    type WorkspaceMapping,
  } from "../lib/activity";
  import {
    categoriesForCatalog,
    findCatalogProject,
    isCatalogReady,
  } from "../lib/rmCatalog";
  import type { RmCatalogCache } from "../lib/rm";

  export let knownWorkspaces: KnownWorkspace[] = [];
  export let mappings: WorkspaceMapping[] = [];
  export let projectSuggestions: string[] = [];
  export let catalog: RmCatalogCache | null = null;
  export let trackingStatus: TrackingStatus | null = null;
  export let trackingSettings: TrackingSettings;
  export let onSaveMapping: (
    workspaceKey: string,
    updates: Partial<WorkspaceMapping>,
  ) => void;
  export let onSaveSettings: (settings: TrackingSettings) => void;

  const editorOptions: EditorKind[] = ["cursor", "code", "vscodium"];

  $: useRmPickers = isCatalogReady(catalog);

  const mappingFor = (workspaceKey: string) =>
    mappings.find((mapping) => mapping.workspaceKey === workspaceKey);

  const projectValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.project ?? "";

  const assignableIdValue = (workspaceKey: string) => {
    const id = mappingFor(workspaceKey)?.assignableId;
    return id != null ? String(id) : "";
  };

  const categoryValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.defaultCategory ?? "";

  const labelValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.label ?? "";

  const ignoredValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.ignored === true;

  const categoryOptionsFor = (_workspaceKey: string) =>
    categoriesForCatalog(catalog);

  const handleProjectChange = (
    workspaceKey: string,
    event: Event,
  ) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    const nextAssignableId = value ? Number(value) : undefined;
    const project = findCatalogProject(catalog, nextAssignableId);

    onSaveMapping(workspaceKey, {
      assignableId: nextAssignableId,
      project: project?.name ?? "",
    });
  };

  type IgnoreFilter = "all" | "hide-ignored" | "only-ignored";
  let ignoreFilter: IgnoreFilter = "all";

  const setIgnoreFilter = (filter: "hide-ignored" | "only-ignored") => {
    ignoreFilter = ignoreFilter === filter ? "all" : filter;
  };

  $: visibleWorkspaces = knownWorkspaces.filter((workspace) => {
    const ignored = ignoredValue(workspace.workspaceKey);

    if (ignoreFilter === "hide-ignored") {
      return !ignored;
    }

    if (ignoreFilter === "only-ignored") {
      return ignored;
    }

    return true;
  });

  const toggleEditor = (editor: EditorKind) => {
    const enabled = new Set(trackingSettings.enabledEditors);
    if (enabled.has(editor)) {
      enabled.delete(editor);
    } else {
      enabled.add(editor);
    }

    onSaveSettings({
      ...trackingSettings,
      enabledEditors: editorOptions.filter((option) => enabled.has(option)),
    });
  };
</script>

<section class="mappings-card glass-card" aria-labelledby="workspaces-heading">
  <div class="entries-header mappings-header">
    <div>
      <p class="eyebrow">Tracking</p>
      <h2 id="workspaces-heading">Workspace mappings</h2>
    </div>
  </div>

  <div id="mappings-panel" class="mappings-panel">
    {#if trackingStatus?.lastError}
      <p class="form-error" role="status">{trackingStatus.lastError}</p>
    {/if}

    <div class="editor-toggles" aria-label="Enabled editors">
      {#each editorOptions as editor}
        <label>
          <input
            type="checkbox"
            checked={trackingSettings.enabledEditors.includes(editor)}
            on:change={() => toggleEditor(editor)}
          />
          <span>{editorLabel(editor)}</span>
        </label>
      {/each}
    </div>

    {#if knownWorkspaces.length === 0}
      <p class="empty-state">
        Open a workspace in Cursor or VS Code while tracking is enabled to build
        this list.
      </p>
    {:else}
      <div class="mapping-filters" aria-label="Workspace list filters">
        <button
          type="button"
          class="filter-btn"
          class:active={ignoreFilter === "hide-ignored"}
          aria-pressed={ignoreFilter === "hide-ignored"}
          on:click={() => setIgnoreFilter("hide-ignored")}
        >
          Hide ignored
        </button>
        <button
          type="button"
          class="filter-btn"
          class:active={ignoreFilter === "only-ignored"}
          aria-pressed={ignoreFilter === "only-ignored"}
          on:click={() => setIgnoreFilter("only-ignored")}
        >
          Show only ignored
        </button>
      </div>

      {#if visibleWorkspaces.length === 0}
        <p class="empty-state">
          {#if ignoreFilter === "only-ignored"}
            No ignored workspaces yet.
          {:else if ignoreFilter === "hide-ignored"}
            All workspaces are ignored. Turn off "Hide ignored" to see them.
          {:else}
            No workspaces to show.
          {/if}
        </p>
      {:else}
      <ul class="mapping-list">
        {#each visibleWorkspaces as workspace (workspace.workspaceKey)}
          <li>
            <div class="mapping-header">
              <div class="mapping-content">
                <strong>{workspace.workspaceLabel}</strong>
                {#if workspace.editor}
                  <span>{editorLabel(workspace.editor)}</span>
                {/if}
              </div>

              <label class="ignore-toggle">
                <input
                  type="checkbox"
                  checked={ignoredValue(workspace.workspaceKey)}
                  aria-label="Ignore suggestions for {workspace.workspaceLabel}"
                  on:change={(event) =>
                    onSaveMapping(workspace.workspaceKey, {
                      ignored: (event.currentTarget as HTMLInputElement).checked,
                    })}
                />
                <span>Ignore</span>
              </label>
            </div>

            <div class="mapping-fields">
              {#if useRmPickers}
                <label>
                  <span>Project</span>
                  <select
                    value={assignableIdValue(workspace.workspaceKey)}
                    on:change={(event) =>
                      handleProjectChange(workspace.workspaceKey, event)}
                  >
                    <option value="">Select a project</option>
                    {#each catalog?.projects ?? [] as rmProject (rmProject.assignableId)}
                      <option value={String(rmProject.assignableId)}>
                        {rmProject.name}
                      </option>
                    {/each}
                  </select>
                </label>

                {#if categoryOptionsFor(workspace.workspaceKey).length > 0}
                  <label>
                    <span>Default category</span>
                    <select
                      value={categoryValue(workspace.workspaceKey)}
                      on:change={(event) =>
                        onSaveMapping(workspace.workspaceKey, {
                          defaultCategory: (
                            event.currentTarget as HTMLSelectElement
                          ).value,
                        })}
                    >
                      <option value="">Select a category</option>
                      {#each categoryOptionsFor(workspace.workspaceKey) as categoryName}
                        <option value={categoryName}>{categoryName}</option>
                      {/each}
                    </select>
                  </label>
                {/if}
              {:else}
                <label>
                  <span>Project</span>
                  <input
                    value={projectValue(workspace.workspaceKey)}
                    list="mapping-project-suggestions"
                    placeholder="NGS, NCC, admin"
                    on:change={(event) =>
                      onSaveMapping(workspace.workspaceKey, {
                        project: (event.currentTarget as HTMLInputElement).value,
                      })}
                  />
                </label>
              {/if}

              <label>
                <span>Label <small>optional</small></span>
                <input
                  value={labelValue(workspace.workspaceKey)}
                  placeholder={workspace.workspaceLabel}
                  on:change={(event) =>
                    onSaveMapping(workspace.workspaceKey, {
                      label: (event.currentTarget as HTMLInputElement).value,
                    })}
                />
              </label>
            </div>
          </li>
        {/each}
      </ul>
      {/if}
    {/if}

    {#if !useRmPickers}
      <datalist id="mapping-project-suggestions">
        {#each projectSuggestions as suggestedProject}
          <option value={suggestedProject}></option>
        {/each}
      </datalist>
    {/if}
  </div>
</section>
