<script lang="ts">
  import {
    entryDuration,
    formatDateLabel,
    formatMinutes,
    loadEntries,
    minutesBetween,
    saveEntries,
    shiftDateKey,
    todayKey,
    totalMinutesForDay,
    type TimeEntry,
  } from "./lib/timeEntries";

  import { Pencil, Trash2 } from "@lucide/svelte";

  let entries: TimeEntry[] = [];
  let selectedDate = todayKey();
  let project = "";
  let startTime = "09:00";
  let endTime = "10:00";
  let notes = "";
  let error = "";
  let isLoading = true;
  let isSaving = false;
  let editingEntryId: string | null = null;

  const DEFAULT_START_TIME = "09:00";
  const DEFAULT_ENTRY_MINUTES = 60;
  const MINUTES_PER_DAY = 24 * 60;

  $: dayEntries = entries
    .filter((entry) => entry.date === selectedDate)
    .sort((a, b) => a.startTime.localeCompare(b.startTime));

  $: totalForSelectedDay = totalMinutesForDay(entries, selectedDate);

  loadEntries()
    .then((storedEntries) => {
      entries = storedEntries;
      applyDefaultTimes();
    })
    .catch((loadError: unknown) => {
      error =
        loadError instanceof Error
          ? loadError.message
          : "Could not load entries.";
    })
    .finally(() => {
      isLoading = false;
    });

  $: isEditing = editingEntryId !== null;

  const submitEntry = async () => {
    error = "";

    const trimmedProject = project.trim();
    const trimmedNotes = notes.trim();

    if (!trimmedProject) {
      error = "Add a project name before saving.";
      return;
    }

    if (!startTime || !endTime) {
      error = "Add both a start and end time.";
      return;
    }

    if (minutesBetween(startTime, endTime) <= 0) {
      error = "End time must be after start time.";
      return;
    }

    const entryValues = {
      date: selectedDate,
      project: trimmedProject,
      startTime,
      endTime,
      notes: trimmedNotes || undefined,
    };

    const nextEntries =
      editingEntryId === null
        ? [
            ...entries,
            {
              id: crypto.randomUUID(),
              ...entryValues,
            },
          ]
        : entries.map((entry) =>
            entry.id === editingEntryId
              ? {
                  ...entry,
                  ...entryValues,
                }
              : entry,
          );

    await persist(nextEntries);

    resetForm();
  };

  const deleteEntry = async (entryId: string) => {
    error = "";
    await persist(entries.filter((entry) => entry.id !== entryId));

    if (editingEntryId === entryId) {
      resetForm();
    } else if (!isEditing) {
      applyDefaultTimes();
    }
  };

  const editEntry = (entry: TimeEntry) => {
    editingEntryId = entry.id;
    selectedDate = entry.date;
    project = entry.project;
    startTime = entry.startTime;
    endTime = entry.endTime;
    notes = entry.notes ?? "";
    error = "";
  };

  const resetForm = () => {
    editingEntryId = null;
    project = "";
    applyDefaultTimes();
    notes = "";
  };

  const moveDay = (days: number) => {
    selectedDate = shiftDateKey(selectedDate, days);
    error = "";

    if (!isEditing) {
      resetForm();
    }
  };

  const jumpToToday = () => {
    selectedDate = todayKey();
    error = "";

    if (!isEditing) {
      resetForm();
    }
  };

  const persist = async (nextEntries: TimeEntry[]) => {
    isSaving = true;

    try {
      await saveEntries(nextEntries);
      entries = nextEntries;
    } catch (saveError) {
      error =
        saveError instanceof Error
          ? saveError.message
          : "Could not save entries.";
    } finally {
      isSaving = false;
    }
  };

  const applyDefaultTimes = () => {
    const nextStartTime = getDefaultStartTime();

    startTime = nextStartTime;
    endTime = addMinutesToTime(nextStartTime, DEFAULT_ENTRY_MINUTES);
  };

  const getDefaultStartTime = () =>
    entries
      .filter((entry) => entry.date === selectedDate)
      .reduce<string | null>(
        (latestEndTime, entry) =>
          latestEndTime === null || entry.endTime > latestEndTime
            ? entry.endTime
            : latestEndTime,
        null,
      ) ?? DEFAULT_START_TIME;

  const addMinutesToTime = (time: string, minutes: number) => {
    const [hours, currentMinutes] = time.split(":").map(Number);
    const totalMinutes =
      (hours * 60 + currentMinutes + minutes) % MINUTES_PER_DAY;

    return `${Math.floor(totalMinutes / 60)}`.padStart(2, "0") +
      `:${totalMinutes % 60}`.padStart(2, "0");
  };
</script>

<main class="app-shell">
  <section class="hero-card" aria-labelledby="app-title">
    <div>
      <h1 id="app-title">Time Tracker</h1>
    </div>

    <div class="day-total">
      <span>Total for day</span>
      <strong>{formatMinutes(totalForSelectedDay)}</strong>
    </div>
  </section>

  <section class="day-card" aria-labelledby="day-heading">
    <div class="day-header">
      <div>
        <p class="eyebrow">Selected day</p>
        <h2 id="day-heading">{formatDateLabel(selectedDate)}</h2>
      </div>

      <div class="day-actions" aria-label="Day navigation">
        <button type="button" on:click={() => moveDay(-1)}>Previous</button>
        <button
          type="button"
          on:click={jumpToToday}
          disabled={selectedDate === todayKey()}
        >
          Today
        </button>
        <button type="button" on:click={() => moveDay(1)}>Next</button>
      </div>
    </div>

    <form
      class:editing={isEditing}
      class="entry-form"
      on:submit|preventDefault={submitEntry}
    >
      {#if isEditing}
        <div class="editing-banner">
          <span>Editing entry</span>
          <button type="button" class="ghost" on:click={resetForm}
            >Cancel</button
          >
        </div>
      {/if}

      <label>
        <span>Project</span>
        <input
          bind:value={project}
          name="project"
          placeholder="Client work, admin, research"
        />
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
        {isSaving ? "Saving..." : isEditing ? "Save changes" : "Add entry"}
      </button>
    </form>
  </section>

  <section class="entries-card" aria-labelledby="entries-heading">
    <div class="entries-header">
      <div>
        <p class="eyebrow">Entries</p>
      </div>
    </div>

    {#if isLoading}
      <p class="empty-state">Loading saved entries...</p>
    {:else if dayEntries.length === 0}
      <p class="empty-state">No time logged for this day yet.</p>
    {:else}
      <ul class="entry-list">
        {#each dayEntries as entry}
          <li>
            <div>
              <div class="entry-title">
                <strong>{entry.project}</strong>
                <span>{formatMinutes(entryDuration(entry))}</span>
              </div>
              <p>{entry.startTime} - {entry.endTime}</p>
              {#if entry.notes}
                <p class="entry-notes">{entry.notes}</p>
              {/if}
            </div>

            <div class="entry-actions">
              <button
                type="button"
                class="ghost"
                on:click={() => editEntry(entry)}><Pencil /></button
              >
              <button
                type="button"
                class="ghost"
                on:click={() => deleteEntry(entry.id)}><Trash2 /></button
              >
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</main>
