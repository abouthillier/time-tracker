<script lang="ts">
  import { CloudUpload, Layers, Pencil, Plus, RefreshCw, Trash2 } from "@lucide/svelte";
  import {
    entryDuration,
    formatDateLabel,
    formatDateSubLabel,
    formatWeekdayLabel,
    formatMinutes,
    minutesBetween,
    todayKey,
    type TimeEntry,
    type TimeSlot,
  } from "../lib/timeEntries";

  export let selectedDate = "";
  export let totalForSelectedDay = 0;
  export let dayEntries: TimeEntry[] = [];
  export let isLoading = false;
  export let canSyncDay = false;
  export let onOpenWorkspaces: () => void;
  export let onOpenRmSettings: () => void;
  export let onSyncDay: () => void;
  export let onMoveDay: (days: number) => void;
  export let onJumpToToday: () => void;
  export let onAddEntry: () => void;
  export let onAddSlotAfter: (entry: TimeEntry, slotIndex: number) => void;
  export let onEditSlot: (entry: TimeEntry, slotIndex: number) => void;
  export let onDeleteSlot: (entryId: string, slotIndex: number) => void;
  export let slotsForDisplay: (entry: TimeEntry) => { slot: TimeSlot; index: number }[];

  const projectInitial = (name: string) =>
    name.trim().charAt(0).toUpperCase() || "?";
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

  <section class="hero-metric" aria-label="Total time for selected day">
    <span class="hero-metric-label">Total for {formatDateLabel(selectedDate)}</span>
    <strong class="hero-metric-value">{formatMinutes(totalForSelectedDay)}</strong>
  </section>

  <section class="day-card glass-card" aria-labelledby="day-heading">
    <div class="day-header">
      <div>
        <p class="eyebrow">Selected day</p>
        <h2 id="day-heading" class="day-heading">
          <span class="day-heading-weekday">{formatWeekdayLabel(selectedDate)}</span>
          <span class="day-heading-date">{formatDateSubLabel(selectedDate)}</span>
        </h2>
      </div>

      <div class="day-actions" aria-label="Day navigation">
        <button type="button" on:click={() => onMoveDay(-1)}>Previous</button>
        <button
          type="button"
          on:click={onJumpToToday}
          disabled={selectedDate === todayKey()}
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
                  {entry.entries.length} slot{entry.entries.length === 1 ? "" : "s"}
                </span>
              </div>

              <span class="transaction-amount entry-project-total">
                {formatMinutes(entryDuration(entry))}
              </span>
            </div>

            <ul class="slot-list transaction-list">
              {#each slotsForDisplay(entry) as displaySlot (displaySlot.index)}
                <li class="transaction-row slot-row">
                  <div class="transaction-body">
                    <span class="transaction-title slot-time">
                      {displaySlot.slot.startTime} – {displaySlot.slot.endTime}
                    </span>
                    {#if displaySlot.slot.notes}
                      <span class="transaction-subtitle">{displaySlot.slot.notes}</span>
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
        {dayEntries.length} group{dayEntries.length === 1 ? "" : "s"} · {formatMinutes(totalForSelectedDay)} total
      </p>
    {/if}
  </section>
</div>
