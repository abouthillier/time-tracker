<script lang="ts">
  import {
    earliestEntryStart,
    entryDuration,
    formatDateLabel,
    formatMinutes,
    latestEntryEnd,
    loadEntries,
    minutesBetween,
    saveEntries,
    shiftDateKey,
    todayKey,
    totalMinutesForDay,
    type TimeEntry,
    type TimeSlot,
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
  let editingSlot: EditingSlot | null = null;

  const DEFAULT_START_TIME = "09:00";
  const DEFAULT_ENTRY_MINUTES = 60;
  const MINUTES_PER_DAY = 24 * 60;

  type EditingSlot = {
    entryId: string;
    slotIndex: number;
  };

  type SlotForDisplay = {
    slot: TimeSlot;
    index: number;
  };

  $: dayEntries = entries
    .filter((entry) => entry.date === selectedDate)
    .sort((a, b) =>
      (earliestEntryStart(a) ?? "").localeCompare(earliestEntryStart(b) ?? ""),
    );

  $: totalForSelectedDay = totalMinutesForDay(entries, selectedDate);

  $: projectSuggestions = entries
    .reduce<string[]>((suggestions, entry) => {
      const hasProject = suggestions.some((suggestion) =>
        isSameProject(suggestion, entry.project),
      );

      return hasProject ? suggestions : [...suggestions, entry.project];
    }, [])
    .sort((a, b) => a.localeCompare(b));

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

  $: isEditing = editingSlot !== null;

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

    const slotValues: TimeSlot = {
      startTime,
      endTime,
      notes: trimmedNotes || undefined,
    };

    const nextEntries =
      editingSlot === null
        ? upsertSlot(entries, selectedDate, trimmedProject, slotValues)
        : moveEditedSlot(
            entries,
            editingSlot,
            selectedDate,
            trimmedProject,
            slotValues,
          );

    await persist(nextEntries);

    resetForm();
  };

  const deleteSlot = async (entryId: string, slotIndex: number) => {
    error = "";
    await persist(removeSlot(entries, entryId, slotIndex));

    if (
      editingSlot?.entryId === entryId &&
      editingSlot.slotIndex === slotIndex
    ) {
      resetForm();
    } else if (
      editingSlot?.entryId === entryId &&
      slotIndex < editingSlot.slotIndex
    ) {
      editingSlot = {
        ...editingSlot,
        slotIndex: editingSlot.slotIndex - 1,
      };
    } else if (!isEditing) {
      applyDefaultTimes();
    }
  };

  const editSlot = (entry: TimeEntry, slotIndex: number) => {
    const slot = entry.entries[slotIndex];
    if (!slot) return;

    editingSlot = { entryId: entry.id, slotIndex };
    selectedDate = entry.date;
    project = entry.project;
    startTime = slot.startTime;
    endTime = slot.endTime;
    notes = slot.notes ?? "";
    error = "";
  };

  const resetForm = () => {
    editingSlot = null;
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
        (currentLatestEndTime, entry) => {
          const entryLatestEndTime = latestEntryEnd(entry);

          return entryLatestEndTime !== null &&
            (currentLatestEndTime === null ||
              entryLatestEndTime > currentLatestEndTime)
            ? entryLatestEndTime
            : currentLatestEndTime;
        },
        null,
      ) ?? DEFAULT_START_TIME;

  const upsertSlot = (
    currentEntries: TimeEntry[],
    date: string,
    projectName: string,
    slot: TimeSlot,
  ) => {
    let didAppendSlot = false;

    const nextEntries = currentEntries.map((entry) => {
      if (entry.date === date && isSameProject(entry.project, projectName)) {
        didAppendSlot = true;

        return {
          ...entry,
          entries: sortSlots([...entry.entries, slot]),
        };
      }

      return entry;
    });

    return didAppendSlot
      ? nextEntries
      : [
          ...nextEntries,
          {
            id: crypto.randomUUID(),
            date,
            project: projectName,
            entries: [slot],
          },
        ];
  };

  const moveEditedSlot = (
    currentEntries: TimeEntry[],
    slotToEdit: EditingSlot,
    date: string,
    projectName: string,
    slot: TimeSlot,
  ) => {
    const originalEntry = currentEntries.find(
      (entry) => entry.id === slotToEdit.entryId,
    );

    if (
      originalEntry &&
      originalEntry.date === date &&
      isSameProject(originalEntry.project, projectName)
    ) {
      return currentEntries.map((entry) =>
        entry.id === slotToEdit.entryId
          ? {
              ...entry,
              project: projectName,
              entries: sortSlots(
                entry.entries.map((existingSlot, index) =>
                  index === slotToEdit.slotIndex ? slot : existingSlot,
                ),
              ),
            }
          : entry,
      );
    }

    return upsertSlot(
      removeSlot(currentEntries, slotToEdit.entryId, slotToEdit.slotIndex),
      date,
      projectName,
      slot,
    );
  };

  const removeSlot = (
    currentEntries: TimeEntry[],
    entryId: string,
    slotIndex: number,
  ) =>
    currentEntries
      .map((entry) =>
        entry.id === entryId
          ? {
              ...entry,
              entries: entry.entries.filter((_, index) => index !== slotIndex),
            }
          : entry,
      )
      .filter((entry) => entry.entries.length > 0);

  const slotsForDisplay = (entry: TimeEntry): SlotForDisplay[] =>
    entry.entries
      .map((slot, index) => ({ slot, index }))
      .sort((a, b) =>
        a.slot.startTime.localeCompare(b.slot.startTime) ||
        a.slot.endTime.localeCompare(b.slot.endTime),
      );

  const sortSlots = (slots: TimeSlot[]) =>
    [...slots].sort(
      (a, b) =>
        a.startTime.localeCompare(b.startTime) ||
        a.endTime.localeCompare(b.endTime),
    );

  const isSameProject = (firstProject: string, secondProject: string) =>
    normalizeProject(firstProject) === normalizeProject(secondProject);

  const normalizeProject = (projectName: string) =>
    projectName.trim().toLocaleLowerCase();

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
          <span>Editing time slot</span>
          <button type="button" class="ghost" on:click={resetForm}
            >Cancel</button
          >
        </div>
      {/if}

      <label>
        <span>Project</span>
        <input
          bind:value={project}
          list="project-suggestions"
          name="project"
          placeholder="Client work, admin, research"
        />
        <datalist id="project-suggestions">
          {#each projectSuggestions as suggestedProject}
            <option value={suggestedProject}></option>
          {/each}
        </datalist>
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
            <div class="entry-content">
              <div class="entry-title">
                <strong>{entry.project}</strong>
                <span>{formatMinutes(entryDuration(entry))}</span>
              </div>

              <ul class="slot-list">
                {#each slotsForDisplay(entry) as displaySlot}
                  <li>
                    <div>
                      <p>
                        {displaySlot.slot.startTime} - {displaySlot.slot.endTime}
                        <span>{formatMinutes(minutesBetween(displaySlot.slot.startTime, displaySlot.slot.endTime))}</span>
                      </p>
                      {#if displaySlot.slot.notes}
                        <p class="entry-notes">{displaySlot.slot.notes}</p>
                      {/if}
                    </div>

                    <div class="entry-actions">
                      <button
                        type="button"
                        class="ghost"
                        on:click={() => editSlot(entry, displaySlot.index)}
                        aria-label={`Edit ${entry.project} time slot`}
                        ><Pencil /></button
                      >
                      <button
                        type="button"
                        class="ghost"
                        on:click={() => deleteSlot(entry.id, displaySlot.index)}
                        aria-label={`Delete ${entry.project} time slot`}
                        ><Trash2 /></button
                      >
                    </div>
                  </li>
                {/each}
              </ul>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</main>
