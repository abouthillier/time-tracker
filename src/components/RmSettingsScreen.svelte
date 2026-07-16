<script lang="ts">
  import { ArrowLeft, RefreshCw } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import {
    clearRmToken,
    findRmUserCandidates,
    formatCatalogAge,
    getRmLinkedUser,
    hasRmToken,
    invokeErrorMessage,
    linkRmUser,
    listenRmCatalogProgress,
    loadRmCatalog,
    loadRmSettings,
    refreshRmCatalog,
    saveRmToken,
    testRmConnection,
    unlinkRmUser,
    type RmCatalogCache,
    type RmCatalogProgress,
    type RmLinkedUser,
    type RmSettings,
    type RmUserSummary,
  } from "../lib/rm";
  import {
    catalogAgeDays,
    categoriesForCatalog,
    isCatalogStale,
  } from "../lib/rmCatalog";

  export let onBack: () => void;
  export let legacyKeyCount = 0;
  export let onOpenMigration: () => void = () => {};

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
  let linkedUser: RmLinkedUser | null = null;
  let identityEmail = "";
  let identityCandidates: RmUserSummary[] = [];
  let isFindingUser = false;
  let isLinkingUser = false;
  let identityStatus = "";
  let identityError = "";

  let unlistenProgress: (() => void) | null = null;

  $: activeToken = tokenInput.trim();
  $: canUseToken = hasToken || activeToken.length > 0;
  $: catalogFetchedAt = settings.catalogFetchedAt ?? catalog?.fetchedAt;
  $: catalogIsStale = Boolean(catalog) && isCatalogStale(catalogFetchedAt);
  $: catalogAge = catalogAgeDays(catalogFetchedAt);
  $: catalogCategories = categoriesForCatalog(catalog);

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
      const [nextSettings, nextCatalog, tokenSaved, nextLinkedUser] =
        await Promise.all([
          loadRmSettings(),
          loadRmCatalog(),
          hasRmToken(),
          getRmLinkedUser(),
        ]);

      settings = nextSettings;
      catalog = nextCatalog;
      hasToken = tokenSaved;
      linkedUser = nextLinkedUser ?? nextSettings.linkedUser ?? null;
      identityEmail = linkedUser?.email ?? identityEmail;
    } catch (loadError) {
      connectionError = invokeErrorMessage(
        loadError,
        "Could not load Resource Management settings.",
      );
    } finally {
      isLoading = false;
    }
  };

  const ensureTokenReady = async () => {
    if (activeToken) {
      await saveRmToken(activeToken);
      hasToken = true;
      return activeToken;
    }

    if (hasToken) {
      return undefined;
    }

    throw new Error(
      "Enter and save an API token before linking your Resource Management user.",
    );
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
      hasToken = await hasRmToken();
      if (!hasToken) {
        throw new Error(
          "Token could not be read back from Windows Credential Manager after saving.",
        );
      }

      await testRmConnection();
      connectionStatus = "Token saved to Windows Credential Manager.";
    } catch (saveError) {
      hasToken = false;
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

  const findIdentityUser = async () => {
    if (!canUseToken) {
      identityError = "Save an API token before linking your RM user.";
      return;
    }

    const trimmedEmail = identityEmail.trim();
    if (!trimmedEmail) {
      identityError = "Enter your work email.";
      return;
    }

    isFindingUser = true;
    identityStatus = "";
    identityError = "";
    identityCandidates = [];

    try {
      const token = await ensureTokenReady();
      const response = await findRmUserCandidates(trimmedEmail, token);
      identityCandidates = response.candidates;

      if (identityCandidates.length === 1) {
        identityStatus = "One match found. Confirm the link below.";
      } else {
        identityStatus = `${identityCandidates.length} matches found. Choose the correct user.`;
      }
    } catch (findError) {
      identityError = invokeErrorMessage(
        findError,
        "Could not find a Resource Management user for that email.",
      );
    } finally {
      isFindingUser = false;
    }
  };

  const confirmLink = async (candidate: RmUserSummary) => {
    isLinkingUser = true;
    identityStatus = "";
    identityError = "";

    try {
      linkedUser = await linkRmUser(
        candidate.id,
        candidate.email,
        candidate.displayName,
      );
      settings = {
        ...settings,
        linkedUser,
      };
      identityCandidates = [];
      identityEmail = linkedUser.email;
      identityStatus = `Linked as ${linkedUser.displayName}.`;
    } catch (linkError) {
      identityError = invokeErrorMessage(
        linkError,
        "Could not link Resource Management user.",
      );
    } finally {
      isLinkingUser = false;
    }
  };

  const removeLinkedUser = async () => {
    identityStatus = "";
    identityError = "";

    try {
      await unlinkRmUser();
      linkedUser = null;
      settings = {
        ...settings,
        linkedUser: undefined,
      };
      identityCandidates = [];
      identityStatus = "Resource Management user unlinked.";
    } catch (unlinkError) {
      identityError = invokeErrorMessage(
        unlinkError,
        "Could not unlink Resource Management user.",
      );
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

    <section class="glass-card rm-panel" aria-labelledby="rm-identity-heading">
      <div class="rm-panel-header">
        <div>
          <p class="eyebrow">Identity</p>
          <h2 id="rm-identity-heading">Your RM user</h2>
        </div>
        {#if linkedUser}
          <span class="rm-status-pill connected">
            Linked as {linkedUser.displayName}
          </span>
        {/if}
      </div>

      <p class="muted-copy">
        Link your work email once so sync always writes to your timesheet. An
        org API token can access other users in RM — prefer a token scoped for
        your own use, and unlink here before changing identity.
      </p>

      {#if linkedUser}
        <p class="rm-identity-linked">
          <strong>{linkedUser.displayName}</strong>
          <span class="muted-copy">{linkedUser.email}</span>
        </p>

        <div class="rm-actions">
          <button
            type="button"
            class="danger-btn"
            disabled={isLinkingUser}
            on:click={removeLinkedUser}
          >
            Unlink
          </button>
        </div>
      {:else}
        <label class="field">
          <span class="field-label">Work email</span>
          <input
            type="email"
            bind:value={identityEmail}
            placeholder="you@company.com"
            autocomplete="email"
          />
        </label>

        <div class="rm-actions">
          <button
            type="button"
            class="primary"
            disabled={!canUseToken || isFindingUser || !identityEmail.trim()}
            on:click={findIdentityUser}
          >
            {isFindingUser ? "Searching…" : "Find my RM user"}
          </button>
        </div>

        {#if identityCandidates.length > 0}
          <ul class="rm-identity-candidates">
            {#each identityCandidates as candidate (candidate.id)}
              <li class="rm-identity-candidate">
                <div>
                  <strong>{candidate.displayName}</strong>
                  <span class="muted-copy">{candidate.email}</span>
                </div>
                <button
                  type="button"
                  disabled={isLinkingUser}
                  on:click={() => confirmLink(candidate)}
                >
                  {isLinkingUser ? "Linking…" : "Confirm link"}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}

      {#if identityStatus}
        <p class="rm-inline-success" role="status">{identityStatus}</p>
      {/if}

      {#if identityError}
        <p class="rm-inline-error" role="alert">{identityError}</p>
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
        {formatCatalogAge(catalogFetchedAt)}
      </p>

      {#if catalogIsStale}
        <p class="rm-stale-warning" role="status">
          Catalog is {catalogAge} days old. Refresh before syncing to pick up
          new projects and categories.
        </p>
      {/if}

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

        {#if legacyKeyCount > 0}
          <button
            type="button"
            class="ghost"
            on:click={onOpenMigration}
          >
            Map local projects ({legacyKeyCount})
          </button>
        {/if}
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

        {#if catalogCategories.length > 0}
          <p class="rm-category-preview">
            Timesheet categories ({catalogCategories.length}):
            {catalogCategories.slice(0, 6).join(" · ")}
            {#if catalogCategories.length > 6}
              · +{catalogCategories.length - 6} more
            {/if}
          </p>
        {:else}
          <p class="rm-category-preview muted-copy">
            No timesheet categories discovered yet. Try refreshing again — we
            probe active projects and RM time entries for category names.
          </p>
        {/if}

        <ul class="rm-project-list">
          {#each catalog.projects as project (project.assignableId)}
            <li class="rm-project-item">
              <div class="rm-project-title">
                <strong>{project.name}</strong>
                {#if project.client}
                  <span class="muted-copy">{project.client}</span>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted-copy">
          Refresh the catalog to load RM projects and the timesheet category list
          used when logging time (for example 3D modeling, Admin, Animation).
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

  .rm-stale-warning {
    margin: 0;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid rgba(251, 191, 36, 0.35);
    background: rgba(251, 191, 36, 0.12);
    color: #fcd34d;
    font-size: 0.9rem;
  }

  .rm-identity-linked {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 0;
  }

  .rm-identity-candidates {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rm-identity-candidate {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--bg-card-border);
    padding-top: 10px;
  }

  .rm-identity-candidate div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
