<script lang="ts">
  import { onMount } from "svelte";
  import { formatDateLabel } from "../lib/timeEntries";
  import {
    invokeErrorMessage,
    syncRmDay,
    type RmLinkedUser,
    type SyncDayResult,
    type SyncPreviewRow,
  } from "../lib/rm";

  export let open = false;
  export let selectedDate = "";
  export let linkedUser: RmLinkedUser | null = null;
  export let onClose: () => void;
  export let onSyncComplete: () => void | Promise<void>;

  type ModalPhase = "loading" | "preview" | "syncing" | "summary" | "error";

  let dialogEl: HTMLDialogElement;
  let previousFocus: HTMLElement | null = null;
  let phase: ModalPhase = "loading";
  let previewRows: SyncPreviewRow[] = [];
  let includedEntryIds = new Set<string>();
  let result: SyncDayResult | null = null;
  let errorMessage = "";
  let wasOpen = false;

  const resetState = () => {
    phase = "loading";
    previewRows = [];
    includedEntryIds = new Set();
    result = null;
    errorMessage = "";
  };

  const loadPreview = async () => {
    phase = "loading";
    errorMessage = "";

    try {
      const previewResult = await syncRmDay({
        date: selectedDate,
        dryRun: true,
        scope: "changed",
      });

      previewRows = previewResult.preview ?? [];
      includedEntryIds = new Set(previewRows.map((row) => row.entryId));

      if (previewRows.length === 0) {
        result = previewResult;
        phase = "summary";
        return;
      }

      phase = "preview";
    } catch (previewError) {
      errorMessage = invokeErrorMessage(
        previewError,
        "Could not preview sync for this day.",
      );
      phase = "error";
    }
  };

  const toggleRow = (entryId: string, included: boolean) => {
    const next = new Set(includedEntryIds);
    if (included) {
      next.add(entryId);
    } else {
      next.delete(entryId);
    }
    includedEntryIds = next;
  };

  const confirmSync = async () => {
    phase = "syncing";
    errorMessage = "";

    const excludeEntryIds = previewRows
      .map((row) => row.entryId)
      .filter((entryId) => !includedEntryIds.has(entryId));

    try {
      result = await syncRmDay({
        date: selectedDate,
        dryRun: false,
        scope: "changed",
        excludeEntryIds,
      });
      await onSyncComplete();
      phase = "summary";
    } catch (syncError) {
      errorMessage = invokeErrorMessage(
        syncError,
        "Could not sync this day to Resource Management.",
      );
      phase = "error";
    }
  };

  const retryFailed = async () => {
    phase = "syncing";
    errorMessage = "";

    try {
      result = await syncRmDay({
        date: selectedDate,
        dryRun: false,
        scope: "failed",
      });
      await onSyncComplete();
      phase = "summary";
    } catch (syncError) {
      errorMessage = invokeErrorMessage(
        syncError,
        "Could not retry failed sync rows.",
      );
      phase = "error";
    }
  };

  const handleBackdropClick = (event: MouseEvent) => {
    if (event.target === dialogEl && phase !== "syncing") {
      onClose();
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && phase !== "syncing") {
      event.preventDefault();
      onClose();
    }
  };

  $: if (dialogEl) {
    if (open && !dialogEl.open) {
      previousFocus = document.activeElement as HTMLElement;
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
      previousFocus?.focus();
      previousFocus = null;
    }
  }

  $: if (open && !wasOpen) {
    wasOpen = true;
    resetState();
    void loadPreview();
  } else if (!open) {
    wasOpen = false;
  }

  onMount(() => {
    return () => {
      if (dialogEl?.open) {
        dialogEl.close();
      }
    };
  });
</script>

<dialog
  bind:this={dialogEl}
  class="entry-modal rm-sync-modal"
  aria-modal="true"
  aria-labelledby="rm-sync-modal-title"
  on:click={handleBackdropClick}
  on:keydown={handleKeydown}
>
  <div class="entry-modal-card rm-sync-card">
    <h2 id="rm-sync-modal-title" class="entry-modal-title">
      Sync day to Resource Management
    </h2>

    <p class="muted-copy rm-sync-subtitle">
      {formatDateLabel(selectedDate)}
      {#if linkedUser}
        · Syncing as {linkedUser.displayName} ({linkedUser.email})
      {/if}
    </p>

    {#if phase === "loading"}
      <p class="muted-copy">Loading preview…</p>
    {:else if phase === "preview"}
      {#if previewRows.length === 0}
        <p class="muted-copy">Nothing to sync for this day.</p>
      {:else}
        <p class="muted-copy">
          Review rows to push. Uncheck any you want to skip.
        </p>

        <ul class="rm-sync-preview-list">
          {#each previewRows as row (row.entryId)}
            <li class="rm-sync-preview-row">
              <label class="rm-sync-preview-label">
                <input
                  type="checkbox"
                  checked={includedEntryIds.has(row.entryId)}
                  on:change={(event) =>
                    toggleRow(
                      row.entryId,
                      (event.currentTarget as HTMLInputElement).checked,
                    )}
                />
                <span class="rm-sync-preview-body">
                  <strong>{row.project}</strong>
                  <span class="muted-copy">
                    {row.category} · {row.hours.toFixed(2)}h · {row.action}
                  </span>
                </span>
              </label>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="rm-sync-actions">
        <button type="button" class="ghost" on:click={onClose}>Cancel</button>
        <button
          type="button"
          class="primary"
          disabled={includedEntryIds.size === 0}
          on:click={confirmSync}
        >
          Confirm sync
        </button>
      </div>
    {:else if phase === "syncing"}
      <p class="muted-copy">Syncing to Resource Management…</p>
    {:else if phase === "summary" && result}
      <p class="rm-inline-success" role="status">
        {result.succeeded} synced, {result.skipped} skipped, {result.failed
          .length}
        failed
      </p>

      {#if result.failed.length > 0}
        <ul class="rm-sync-failure-list">
          {#each result.failed as failure (failure.entryId)}
            <li class="rm-sync-failure-item">
              <strong>{failure.project}</strong>
              <span class="muted-copy">{failure.error}</span>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="rm-sync-actions">
        {#if result.failed.length > 0}
          <button type="button" class="primary" on:click={retryFailed}>
            Retry failed
          </button>
        {/if}
        <button type="button" class="ghost" on:click={onClose}>Close</button>
      </div>
    {:else if phase === "error"}
      <p class="rm-inline-error" role="alert">{errorMessage}</p>
      <div class="rm-sync-actions">
        <button type="button" class="ghost" on:click={onClose}>Close</button>
        <button type="button" class="primary" on:click={loadPreview}>
          Try again
        </button>
      </div>
    {/if}
  </div>
</dialog>

<style>
  .rm-sync-modal {
    max-width: min(520px, calc(100vw - 32px));
  }

  .rm-sync-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .rm-sync-subtitle {
    margin: -8px 0 0;
  }

  .rm-sync-preview-list,
  .rm-sync-failure-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 280px;
    overflow-y: auto;
  }

  .rm-sync-preview-row,
  .rm-sync-failure-item {
    border-top: 1px solid var(--bg-card-border);
    padding-top: 10px;
  }

  .rm-sync-preview-label {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    cursor: pointer;

    input {
      width: unset;
    }
  }

  .rm-sync-preview-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .rm-sync-failure-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .rm-sync-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: flex-end;
  }

  .rm-inline-success {
    margin: 0;
    color: #8fd49b;
  }

  .rm-inline-error {
    margin: 0;
    border-radius: 12px;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
    padding: 10px 12px;
  }
</style>
