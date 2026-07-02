<script lang="ts">
  import { ArrowLeft, RefreshCw } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import {
    clearRmToken,
    formatCatalogAge,
    hasRmToken,
    invokeErrorMessage,
    listenRmCatalogProgress,
    loadRmCatalog,
    loadRmSettings,
    refreshRmCatalog,
    saveRmToken,
    testRmConnection,
    type RmCatalogCache,
    type RmCatalogProgress,
    type RmSettings,
  } from "../lib/rm";

  export let onBack: () => void;

  let settings: RmSettings = { region: "us" };
  let catalog: RmCatalogCache | null = null;
  let tokenInput = "";
  let hasToken = false;
  let isLoading = true;
  let isSavingToken = false;
  let isTesting = false;
  let isRefreshing = false;
  let connectionStatus = "";
  let connectionError = "";
  let catalogStatus = "";
  let catalogError = "";
  let progress: RmCatalogProgress | null = null;

  let unlistenProgress: (() => void) | null = null;

  $: activeToken = tokenInput.trim();
  $: canUseToken = hasToken || activeToken.length > 0;

  const clearFeedback = () => {
    connectionStatus = "";
    connectionError = "";
    catalogStatus = "";
    catalogError = "";
  };

  const tokenForActions = () => activeToken || undefined;

  const loadState = async () => {
    isLoading = true;
    clearFeedback();

    try {
      const [nextSettings, nextCatalog, tokenSaved] = await Promise.all([
        loadRmSettings(),
        loadRmCatalog(),
        hasRmToken(),
      ]);

      settings = nextSettings;
      catalog = nextCatalog;
      hasToken = tokenSaved;
    } catch (loadError) {
      connectionError = invokeErrorMessage(
        loadError,
        "Could not load Resource Management settings.",
      );
    } finally {
      isLoading = false;
    }
  };

  const saveToken = async () => {
    if (!activeToken) {
      connectionError = "Enter an API token before saving.";
      return;
    }

    isSavingToken = true;
    connectionStatus = "";
    connectionError = "";

    try {
      await saveRmToken(activeToken);
      hasToken = true;
      connectionStatus = "Token saved to Windows Credential Manager.";
    } catch (saveError) {
      connectionError = invokeErrorMessage(
        saveError,
        "Could not save API token.",
      );
    } finally {
      isSavingToken = false;
    }
  };

  const removeToken = async () => {
    connectionStatus = "";
    connectionError = "";

    try {
      await clearRmToken();
      hasToken = false;
      tokenInput = "";
      connectionStatus = "API token removed.";
    } catch (removeError) {
      connectionError = invokeErrorMessage(
        removeError,
        "Could not remove API token.",
      );
    }
  };

  const testConnection = async () => {
    if (!canUseToken) {
      connectionError = "Enter or save an API token before testing.";
      return;
    }

    isTesting = true;
    connectionStatus = "";
    connectionError = "";

    try {
      await testRmConnection(tokenForActions());
      connectionStatus = "Connected to Resource Management (US).";
    } catch (testError) {
      connectionError = invokeErrorMessage(
        testError,
        "Connection test failed.",
      );
    } finally {
      isTesting = false;
    }
  };

  const refreshCatalog = async () => {
    if (!canUseToken) {
      catalogError = "Enter or save an API token before refreshing.";
      return;
    }

    isRefreshing = true;
    catalogStatus = "";
    catalogError = "";
    progress = null;

    try {
      catalog = await refreshRmCatalog(tokenForActions());
      settings = {
        ...settings,
        catalogFetchedAt: catalog.fetchedAt,
      };
      catalogStatus = `Cached ${catalog.projects.length} projects from Resource Management.`;
    } catch (refreshError) {
      catalogError = invokeErrorMessage(
        refreshError,
        "Could not refresh project catalog.",
      );
    } finally {
      isRefreshing = false;
      progress = null;
    }
  };

  onMount(async () => {
    try {
      unlistenProgress = await listenRmCatalogProgress((nextProgress) => {
        progress = nextProgress;
      });
    } catch (listenError) {
      console.warn("RM progress listener unavailable:", listenError);
    }

    await loadState();
  });

  onDestroy(() => {
    unlistenProgress?.();
  });
</script>

<div class="screen rm-settings-screen">
  <header class="screen-header workspaces-header">
    <button
      type="button"
      class="icon-btn"
      aria-label="Back to dashboard"
      on:click={onBack}
    >
      <ArrowLeft />
    </button>

    <h1>Resource Management</h1>
  </header>

  {#if isLoading}
    <p class="muted-copy">Loading settings…</p>
  {:else}
    <section
      class="glass-card rm-panel"
      aria-labelledby="rm-connection-heading"
    >
      <div class="rm-panel-header">
        <div>
          <p class="eyebrow">Connection</p>
          <h2 id="rm-connection-heading">API token</h2>
        </div>
        <span class="rm-status-pill" class:connected={hasToken}>
          {hasToken ? "Saved" : "Not configured"}
        </span>
      </div>

      <p class="muted-copy">
        Token is stored in Windows Credential Manager, not in your time entry
        files. You can test with a pasted token before saving.
      </p>

      <label class="field">
        <span class="field-label">API token</span>
        <input
          type="password"
          bind:value={tokenInput}
          placeholder={hasToken
            ? "Enter a new token to replace"
            : "Paste RM API token"}
          autocomplete="off"
        />
      </label>

      <div class="rm-actions">
        <button
          type="button"
          class="primary"
          disabled={isSavingToken || !activeToken}
          on:click={saveToken}
        >
          {isSavingToken ? "Saving…" : "Save token"}
        </button>
        <button
          type="button"
          disabled={!canUseToken || isTesting}
          on:click={testConnection}
        >
          {isTesting ? "Testing…" : "Test connection"}
        </button>
        {#if hasToken}
          <button type="button" class="danger-btn" on:click={removeToken}>
            Remove token
          </button>
        {/if}
      </div>

      {#if connectionStatus}
        <p class="rm-inline-success" role="status">{connectionStatus}</p>
      {/if}

      {#if connectionError}
        <p class="rm-inline-error" role="alert">{connectionError}</p>
      {/if}
    </section>

    <section class="glass-card rm-panel" aria-labelledby="rm-catalog-heading">
      <div class="rm-panel-header">
        <div>
          <p class="eyebrow">Catalog</p>
          <h2 id="rm-catalog-heading">Projects and categories</h2>
        </div>
      </div>

      <p class="muted-copy">
        Last refreshed:
        {formatCatalogAge(settings.catalogFetchedAt ?? catalog?.fetchedAt)}
      </p>

      {#if progress}
        <p class="rm-progress" aria-live="polite">
          {progress.message}
          {#if progress.total > 0}
            ({progress.current}/{progress.total})
          {/if}
        </p>
      {/if}

      <div class="rm-actions">
        <button
          type="button"
          class="primary"
          disabled={!canUseToken || isRefreshing}
          on:click={refreshCatalog}
        >
          <span class:spin={isRefreshing}>
            <RefreshCw />
          </span>
          {isRefreshing ? "Refreshing…" : "Refresh from RM"}
        </button>
      </div>

      {#if catalogStatus}
        <p class="rm-inline-success" role="status">{catalogStatus}</p>
      {/if}

      {#if catalogError}
        <p class="rm-inline-error" role="alert">{catalogError}</p>
      {/if}

      {#if catalog}
        <p class="rm-catalog-summary">
          {catalog.projects.length} projects cached locally.
        </p>

        <ul class="rm-project-list">
          {#each catalog.projects as project (project.assignableId)}
            <li class="rm-project-item">
              <div class="rm-project-title">
                <strong>{project.name}</strong>
                {#if project.client}
                  <span class="muted-copy">{project.client}</span>
                {/if}
              </div>
              <p class="rm-category-preview">
                {#if project.categories.length > 0}
                  {project.categories.slice(0, 4).join(" · ")}
                  {#if project.categories.length > 4}
                    · +{project.categories.length - 4} more
                  {/if}
                {:else}
                  No categories returned
                {/if}
              </p>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted-copy">
          Refresh the catalog to load RM projects and their time-entry
          categories.
        </p>
      {/if}
    </section>
  {/if}
</div>

<style>
  .rm-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
  }

  .rm-panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .rm-status-pill {
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
  }

  .rm-status-pill.connected {
    background: rgba(77, 124, 255, 0.18);
    color: var(--text-primary);
  }

  .rm-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .rm-actions .primary {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .rm-actions .spin {
    display: inline-flex;
    animation: spin 0.9s linear infinite;
  }

  .danger-btn {
    color: #ff8f8f;
  }

  .rm-progress,
  .rm-catalog-summary,
  .rm-inline-success,
  .rm-inline-error {
    margin: 0;
    font-size: 0.9rem;
  }

  .rm-inline-success {
    color: #8fd49b;
  }

  .rm-inline-error {
    border-radius: 12px;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
    padding: 10px 12px;
  }

  .rm-project-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 280px;
    overflow-y: auto;
  }

  .rm-project-item {
    border-top: 1px solid var(--bg-card-border);
    padding-top: 10px;
  }

  .rm-project-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .rm-category-preview {
    margin: 6px 0 0;
    font-size: 0.82rem;
    color: var(--text-muted);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
