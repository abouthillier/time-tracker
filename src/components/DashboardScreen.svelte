<script lang="ts">
  import {
    AlertCircle,
    CheckCircle2,
    Circle,
    CloudUpload,
    Copy,
    Layers,
    Pencil,
    Plus,
    RefreshCw,
    Trash2,
  } from "@lucide/svelte";
  import {
    dayEntryGaps,
    entryDuration,
    entryRmSyncStatus,
    entrySyncStatusLabel,
    formatDateLabel,
    formatDateSubLabel,
    formatTimeGap,
    formatWeekdayLabel,
    formatMinutes,
    minutesBetween,
    todayKey,
    type RmEntrySyncStatus,
    type TimeEntry,
    type TimeSlot,
  } from "../lib/timeEntries";

  export let selectedDate = "";
  export let totalForSelectedDay = 0;
  export let dayEntries: TimeEntry[] = [];
  export let isLoading = false;
  export let canSyncDay = false;
  export let rmSyncEnabled = false;
  export let catalogIsStale = false;
  export let retryingEntryId: string | null = null;
  export let rowSyncError = "";
  export let onOpenWorkspaces: () => void;
  export let onOpenRmSettings: () => void;
  export let onSyncDay: () => void;
  export let onMoveDay: (days: number) => void;
  export let onJumpToToday: () => void;
  export let onAddEntry: () => void;
  export let onAddSlotAfter: (entry: TimeEntry, slotIndex: number) => void;
  export let onEditSlot: (entry: TimeEntry, slotIndex: number) => void;
  export let onDuplicateSlotForToday: (
    entry: TimeEntry,
    slotIndex: number,
  ) => void;
  export let onDeleteSlot: (entryId: string, slotIndex: number) => void;
  export let onRetryEntrySync: (entry: TimeEntry) => void;
  export let onMarkEntryResolved: (entry: TimeEntry) => void;
  export let onClearEntryRemoteLink: (entry: TimeEntry) => void;
  export let slotsForDisplay: (
    entry: TimeEntry,
  ) => { slot: TimeSlot; index: number }[];

  $: isViewingToday = selectedDate === todayKey();
  $: dayGaps = dayEntryGaps(dayEntries);
  $: gapsHeading = isViewingToday
    ? "Gaps in today's entries:"
    : "Gaps in this day's entries:";

  const projectInitial = (name: string) =>
    name.trim().charAt(0).toUpperCase() || "?";

  const syncIconTitle = (entry: TimeEntry, status: RmEntrySyncStatus) => {
    const label = entrySyncStatusLabel(status);
    const error = entry.rmSync?.lastError?.trim();
    if (status === "error" && error) {
      return `${label}: ${error}`;
    }
    if (status === "synced" && entry.rmSync?.lastSyncedAt) {
      return `${label} at ${new Intl.DateTimeFormat(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date(entry.rmSync.lastSyncedAt))}`;
    }
    return label;
  };

  const showRowActions = (status: RmEntrySyncStatus | null) =>
    status === "error" || status === "dirty";

  const confirmMarkResolved = (entry: TimeEntry) => {
    if (
      window.confirm(
        `Mark "${entry.project}" as resolved? Use this if you fixed the row directly in Resource Management.`,
      )
    ) {
      onMarkEntryResolved(entry);
    }
  };

  const confirmClearLink = (entry: TimeEntry) => {
    if (
      window.confirm(
        `Clear the RM link for "${entry.project}"? The next sync will create a new RM time entry.`,
      )
    ) {
      onClearEntryRemoteLink(entry);
    }
  };
</script>

<div class="screen dashboard-screen">
  <header class="screen-header">
    <h1 id="app-title">Time Tracker</h1>
    <div class="header-actions">
      <button
        type="button"
        class="icon-btn"
        aria-label="Open Resource Management settings"
        on:click={onOpenRmSettings}
      >
        <CloudUpload />
      </button>
      <button
        type="button"
        class="icon-btn"
        aria-label="Open workspaces"
        on:click={onOpenWorkspaces}
      >
        <Layers />
      </button>
    </div>
  </header>

  {#if catalogIsStale}
    <p class="rm-stale-banner" role="status">
      RM catalog is over 7 days old.
      <button type="button" class="link-btn" on:click={onOpenRmSettings}>
        Refresh in settings
      </button>
    </p>
  {/if}

  {#if rowSyncError}
    <p class="rm-row-sync-error" role="alert">{rowSyncError}</p>
  {/if}

  <section class="hero-metric" aria-label="Total time for selected day">
    <span class="hero-metric-label"
      >Total for {formatDateLabel(selectedDate)}</span
    >
    <strong class="hero-metric-value"
      >{formatMinutes(totalForSelectedDay)}</strong
    >
  </section>

  <section class="day-card glass-card" aria-labelledby="day-heading">
    <div class="day-header">
      <div>
        <p class="eyebrow">Selected day</p>
        <h2 id="day-heading" class="day-heading">
          <span class="day-heading-weekday"
            >{formatWeekdayLabel(selectedDate)}</span
          >
          <span class="day-heading-date"
            >{formatDateSubLabel(selectedDate)}</span
          >
        </h2>
      </div>

      <div class="day-actions" aria-label="Day navigation">
        <button type="button" on:click={() => onMoveDay(-1)}>Previous</button>
        <button
          type="button"
          on:click={onJumpToToday}
          disabled={isViewingToday}
        >
          Today
        </button>
        <button type="button" on:click={() => onMoveDay(1)}>Next</button>
      </div>
    </div>

    <button type="button" class="primary add-entry-btn" on:click={onAddEntry}>
      <Plus />
      Add entry
    </button>

    <button
      type="button"
      class="ghost sync-day-btn"
      disabled={!canSyncDay}
      on:click={onSyncDay}
    >
      <RefreshCw />
      Sync day to RM
    </button>
  </section>

  {#if dayEntries.length > 0 && dayGaps.length > 0}
    <section
      class="day-gaps-card glass-card"
      aria-labelledby="day-gaps-heading"
    >
      <div class="day-gaps-banner" role="status">
        <p class="day-gaps-heading">{gapsHeading}</p>
        <ul class="day-gaps-list">
          {#each dayGaps as gap (gap.startTime + gap.endTime)}
            <li>{formatTimeGap(gap)}</li>
          {/each}
        </ul>
      </div>
    </section>
  {:else if dayEntries.length > 0 && dayGaps.length === 0}
    <p class="empty-state">No gaps in today's entries.</p>
  {:else if dayEntries.length === 0 && dayGaps.length > 0}
    <p class="empty-state">No entries for this day.</p>
  {:else}
    <p class="empty-state">No entries or gaps for this day.</p>
  {/if}

  <section class="entries-card glass-card" aria-labelledby="entries-heading">
    <div class="entries-header">
      <p class="eyebrow" id="entries-heading">Entries</p>
    </div>

    {#if isLoading}
      <p class="empty-state">Loading saved entries...</p>
    {:else if dayEntries.length === 0}
      <p class="empty-state">No time logged for this day yet.</p>
    {:else}
      <ul class="entry-list">
        {#each dayEntries as entry (entry.id)}
          {@const syncStatus = rmSyncEnabled ? entryRmSyncStatus(entry) : null}
          <li class="entry-project-group">
            <div class="entry-project-header">
              <span class="transaction-avatar" aria-hidden="true">
                {projectInitial(entry.project)}
              </span>

              <div class="transaction-body">
                <strong class="transaction-title">{entry.project}</strong>
                <span class="transaction-subtitle">
                  {#if entry.category}
                    {entry.category} ·
                  {/if}
                  {entry.entries.length} slot{entry.entries.length === 1
                    ? ""
                    : "s"}
                </span>
                {#if syncStatus === "error" && entry.rmSync?.lastError}
                  <span class="entry-sync-error">{entry.rmSync.lastError}</span>
                {/if}
              </div>

              {#if syncStatus}
                <span
                  class="entry-sync-badge"
                  class:synced={syncStatus === "synced"}
                  class:dirty={syncStatus === "dirty" ||
                    syncStatus === "pending"}
                  class:error={syncStatus === "error"}
                  title={syncIconTitle(entry, syncStatus)}
                  aria-label={syncIconTitle(entry, syncStatus)}
                >
                  {#if syncStatus === "synced"}
                    <CheckCircle2 />
                  {:else if syncStatus === "error"}
                    <AlertCircle />
                  {:else if syncStatus === "dirty"}
                    <RefreshCw />
                  {:else}
                    <Circle />
                  {/if}
                </span>
              {/if}

              <span class="transaction-amount entry-project-total">
                {formatMinutes(entryDuration(entry))}
              </span>
            </div>

            {#if syncStatus && showRowActions(syncStatus)}
              <div class="entry-sync-actions">
                <button
                  type="button"
                  class="ghost"
                  disabled={retryingEntryId === entry.id}
                  on:click={() => onRetryEntrySync(entry)}
                >
                  {retryingEntryId === entry.id ? "Retrying…" : "Retry"}
                </button>
                {#if syncStatus === "error"}
                  <button
                    type="button"
                    class="ghost"
                    on:click={() => confirmMarkResolved(entry)}
                  >
                    Mark resolved
                  </button>
                  <button
                    type="button"
                    class="ghost"
                    on:click={() => confirmClearLink(entry)}
                  >
                    Clear RM link
                  </button>
                {/if}
              </div>
            {/if}

            <ul class="slot-list transaction-list">
              {#each slotsForDisplay(entry) as displaySlot (displaySlot.index)}
                <li class="transaction-row slot-row">
                  <div class="transaction-body">
                    <span class="transaction-title slot-time">
                      {displaySlot.slot.startTime} – {displaySlot.slot.endTime}
                    </span>
                    {#if displaySlot.slot.notes}
                      <span class="transaction-subtitle"
                        >{displaySlot.slot.notes}</span
                      >
                    {/if}
                  </div>

                  <span class="transaction-amount">
                    {formatMinutes(
                      minutesBetween(
                        displaySlot.slot.startTime,
                        displaySlot.slot.endTime,
                      ),
                    )}
                  </span>

                  <div class="entry-actions">
                    <button
                      type="button"
                      class="ghost icon-btn-sm"
                      on:click={() => onEditSlot(entry, displaySlot.index)}
                      aria-label={`Edit ${entry.project} time slot`}
                    >
                      <Pencil />
                    </button>
                    <button
                      type="button"
                      class="ghost icon-btn-sm"
                      on:click={() => onAddSlotAfter(entry, displaySlot.index)}
                      aria-label={`Add ${entry.project} time slot after this one`}
                    >
                      <Plus />
                    </button>
                    {#if !isViewingToday}
                      <button
                        type="button"
                        class="ghost icon-btn-sm"
                        on:click={() =>
                          onDuplicateSlotForToday(entry, displaySlot.index)}
                        aria-label={`Duplicate ${entry.project} time slot for today`}
                        title="Duplicate for today"
                      >
                        <Copy />
                      </button>
                    {/if}
                    <button
                      type="button"
                      class="ghost icon-btn-sm"
                      on:click={() => onDeleteSlot(entry.id, displaySlot.index)}
                      aria-label={`Delete ${entry.project} time slot`}
                    >
                      <Trash2 />
                    </button>
                  </div>
                </li>
              {/each}
            </ul>
          </li>
        {/each}
      </ul>

      <p class="entries-total">
        {dayEntries.length} group{dayEntries.length === 1 ? "" : "s"} · {formatMinutes(
          totalForSelectedDay,
        )} total
      </p>
    {/if}
  </section>
</div>

<style>
  .rm-stale-banner {
    margin: 0 0 12px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid rgba(251, 191, 36, 0.35);
    background: rgba(251, 191, 36, 0.12);
    color: #fcd34d;
    font-size: 0.9rem;
  }

  .day-gaps-banner {
    margin: 0 0 12px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid rgba(251, 191, 36, 0.35);
    background: rgba(251, 191, 36, 0.12);
    color: #fcd34d;
    font-size: 0.9rem;
  }

  .day-gaps-heading {
    margin: 0 0 4px;
    font-weight: 600;
  }

  .day-gaps-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 2px;
  }

  .rm-row-sync-error {
    margin: 0 0 12px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid rgba(239, 68, 68, 0.3);
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    font-size: 0.9rem;
  }

  .link-btn {
    border: none;
    background: none;
    color: inherit;
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
    font: inherit;
  }

  .entry-sync-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-muted);
  }

  .entry-sync-badge :global(svg) {
    width: 16px;
    height: 16px;
  }

  .entry-sync-badge.synced {
    color: #8fd49b;
    background: rgba(143, 212, 155, 0.12);
  }

  .entry-sync-badge.dirty {
    color: #93c5fd;
    background: rgba(147, 197, 253, 0.12);
  }

  .entry-sync-badge.error {
    color: #fca5a5;
    background: rgba(239, 68, 68, 0.15);
  }

  .entry-sync-error {
    display: block;
    margin-top: 4px;
    color: #fca5a5;
    font-size: 0.82rem;
    line-height: 1.35;
  }

  .entry-sync-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 0 0 8px 48px;
  }
</style>
