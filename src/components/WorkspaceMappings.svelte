<script lang="ts">
  import {
    editorLabel,
    type EditorKind,
    type KnownWorkspace,
    type TrackingSettings,
    type TrackingStatus,
    type WorkspaceMapping,
  } from "../lib/activity";
  import { ChevronDown } from "@lucide/svelte";

  export let knownWorkspaces: KnownWorkspace[] = [];
  export let mappings: WorkspaceMapping[] = [];
  export let projectSuggestions: string[] = [];
  export let trackingStatus: TrackingStatus | null = null;
  export let trackingSettings: TrackingSettings;
  export let onToggleTracking: (enabled: boolean) => void;
  export let onSaveMapping: (
    workspaceKey: string,
    updates: Partial<WorkspaceMapping>,
  ) => void;
  export let onSaveSettings: (settings: TrackingSettings) => void;

  let expanded = false;

  const editorOptions: EditorKind[] = ["cursor", "code", "vscodium"];

  const mappingFor = (workspaceKey: string) =>
    mappings.find((mapping) => mapping.workspaceKey === workspaceKey);

  const projectValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.project ?? "";

  const labelValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.label ?? "";

  const ignoredValue = (workspaceKey: string) =>
    mappingFor(workspaceKey)?.ignored === true;

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

<section class="mappings-card" class:collapsed={!expanded} aria-labelledby="workspaces-heading">
  <div class="entries-header mappings-header">
    <button
      type="button"
      class="collapse-toggle"
      aria-expanded={expanded}
      aria-controls="mappings-panel"
      on:click={() => (expanded = !expanded)}
    >
      <span class="collapse-icon" class:expanded>
        <ChevronDown />
      </span>
      <span>
        <p class="eyebrow">Tracking</p>
        <h2 id="workspaces-heading">Workspace mappings</h2>
      </span>
    </button>

    <label class="tracking-toggle">
      <input
        type="checkbox"
        checked={trackingStatus?.enabled ?? false}
        on:change={(event) =>
          onToggleTracking((event.currentTarget as HTMLInputElement).checked)}
      />
      <span>Track editor focus</span>
    </label>
  </div>

  <div id="mappings-panel" class="mappings-panel" hidden={!expanded}>
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
      <ul class="mapping-list">
        {#each knownWorkspaces as workspace (workspace.workspaceKey)}
          <li>
            <div class="mapping-content">
              <strong>{workspace.workspaceLabel}</strong>
              {#if workspace.editor}
                <span>{editorLabel(workspace.editor)}</span>
              {/if}
            </div>

            <div class="mapping-fields">
              <label>
                <span>Project</span>
                <input
                  value={projectValue(workspace.workspaceKey)}
                  list="project-suggestions"
                  placeholder="NGS, NCC, admin"
                  on:change={(event) =>
                    onSaveMapping(workspace.workspaceKey, {
                      project: (event.currentTarget as HTMLInputElement).value,
                    })}
                />
              </label>

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

              <label class="ignore-toggle">
                <input
                  type="checkbox"
                  checked={ignoredValue(workspace.workspaceKey)}
                  on:change={(event) =>
                    onSaveMapping(workspace.workspaceKey, {
                      ignored: (event.currentTarget as HTMLInputElement).checked,
                    })}
                />
                <span>Ignore suggestions</span>
              </label>
            </div>
          </li>
        {/each}
      </ul>
    {/if}

    <datalist id="project-suggestions">
      {#each projectSuggestions as suggestedProject}
        <option value={suggestedProject}></option>
      {/each}
    </datalist>
  </div>
</section>
