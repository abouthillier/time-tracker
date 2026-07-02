<script lang="ts">
  import {
    earliestEntryStart,
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

  import {
    getTrackingStatus,
    listKnownWorkspaces,
    loadActivitySegments,
    loadSuggestionState,
    loadTrackingSettings,
    loadWorkspaceMappings,
    saveSuggestionState,
    saveTrackingSettings,
    saveWorkspaceMappings,
    startActivityTracking,
    stopActivityTracking,
    type ActivitySegment,
    type KnownWorkspace,
    type SuggestionStateEntry,
    type TrackingSettings,
    type TrackingStatus,
    type WorkspaceMapping,
  } from "./lib/activity";

  import {
    buildActivitySuggestions,
    type ActivitySuggestion,
  } from "./lib/activitySuggestions";

  import DashboardScreen from "./components/DashboardScreen.svelte";
  import EntryFormModal from "./components/EntryFormModal.svelte";
  import RmSettingsScreen from "./components/RmSettingsScreen.svelte";
  import WorkspacesScreen from "./components/WorkspacesScreen.svelte";

  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  type ActiveScreen = "dashboard" | "workspaces" | "rm";

  type EditingSlot = {
    entryId: string;
    slotIndex: number;
  };

  type SlotForDisplay = {
    slot: TimeSlot;
    index: number;
  };

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
  let activitySegments: ActivitySegment[] = [];
  let workspaceMappings: WorkspaceMapping[] = [];
  let suggestionState: SuggestionStateEntry[] = [];
  let knownWorkspaces: KnownWorkspace[] = [];
  let trackingStatus: TrackingStatus | null = null;
  let trackingSettings: TrackingSettings = {
    enabledEditors: ["cursor", "code", "vscodium"],
    pollIntervalSecs: 10,
  };
  let activityError = "";
  let activeScreen: ActiveScreen = "dashboard";
  let entryModalOpen = false;

  let unlistenSegments: (() => void) | null = null;
  let pendingSuggestion: ActivitySuggestion | null = null;

  const DEFAULT_START_TIME = "09:00";
  const DEFAULT_ENTRY_MINUTES = 60;
  const MINUTES_PER_DAY = 24 * 60;

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

  $: activitySuggestions = buildActivitySuggestions({
    segments: activitySegments,
    mappings: workspaceMappings,
    suggestionState,
    entries,
    dateKey: selectedDate,
  });

  $: isEditing = editingSlot !== null;

  const refreshActivityData = async () => {
    const [segments, mappings, state, workspaces, status, settings] =
      await Promise.all([
        loadActivitySegments(),
        loadWorkspaceMappings(),
        loadSuggestionState(),
        listKnownWorkspaces(),
        getTrackingStatus(),
        loadTrackingSettings(),
      ]);

    activitySegments = segments;
    workspaceMappings = mappings;
    suggestionState = state;
    knownWorkspaces = workspaces;
    trackingStatus = status;
    trackingSettings = settings;
  };

  onMount(() => {
    const setup = async () => {
      try {
        await refreshActivityData();
        unlistenSegments = await listen<ActivitySegment[]>(
          "activity-segment-updated",
          (event) => {
            activitySegments = event.payload;
          },
        );
      } catch (loadError: unknown) {
        activityError =
          loadError instanceof Error
            ? loadError.message
            : "Could not load activity tracking.";
      }
    };

    void setup();
  });

  onDestroy(() => {
    unlistenSegments?.();
  });

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

  const openEntryModal = () => {
    entryModalOpen = true;
  };

  const closeEntryModal = () => {
    entryModalOpen = false;
    resetForm();
  };

  const openAddEntry = () => {
    resetForm();
    openEntryModal();
  };

  const addSlotAfter = (entry: TimeEntry, slotIndex: number) => {
    const slot = entry.entries[slotIndex];
    if (!slot) return;

    editingSlot = null;
    pendingSuggestion = null;
    selectedDate = entry.date;
    project = entry.project;
    startTime = slot.endTime;
    endTime = addMinutesToTime(slot.endTime, DEFAULT_ENTRY_MINUTES);
    notes = "";
    error = "";
    openEntryModal();
  };

  const openWorkspaces = () => {
    activeScreen = "workspaces";
  };

  const closeWorkspaces = () => {
    activeScreen = "dashboard";
  };

  const openRmSettings = () => {
    activeScreen = "rm";
  };

  const closeRmSettings = () => {
    activeScreen = "dashboard";
  };

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

    if (pendingSuggestion) {
      const suggestion = pendingSuggestion;
      const existingMapping = workspaceMappings.find(
        (mapping) => mapping.workspaceKey === suggestion.workspaceKey,
      );

      if (!existingMapping?.project?.trim()) {
        await upsertWorkspaceMapping(suggestion.workspaceKey, {
          project: trimmedProject,
        });
      }

      await finalizeSuggestion(suggestion);
    }

    closeEntryModal();
  };

  const deleteSlot = async (entryId: string, slotIndex: number) => {
    error = "";
    await persist(removeSlot(entries, entryId, slotIndex));

    if (
      editingSlot?.entryId === entryId &&
      editingSlot.slotIndex === slotIndex
    ) {
      closeEntryModal();
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
    openEntryModal();
  };

  const resetForm = () => {
    editingSlot = null;
    pendingSuggestion = null;
    project = "";
    applyDefaultTimes();
    notes = "";
    error = "";
  };

  const moveDay = (days: number) => {
    selectedDate = shiftDateKey(selectedDate, days);

    if (!isEditing) {
      if (entryModalOpen) {
        closeEntryModal();
      } else {
        resetForm();
      }
    }
  };

  const jumpToToday = () => {
    selectedDate = todayKey();

    if (!isEditing) {
      if (entryModalOpen) {
        closeEntryModal();
      } else {
        resetForm();
      }
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

  const upsertWorkspaceMapping = async (
    workspaceKey: string,
    updates: Partial<WorkspaceMapping>,
  ): Promise<boolean> => {
    activityError = "";

    const existing = workspaceMappings.find(
      (mapping) => mapping.workspaceKey === workspaceKey,
    );
    const workspace = knownWorkspaces.find(
      (item) => item.workspaceKey === workspaceKey,
    );

    const nextMapping: WorkspaceMapping = {
      workspaceKey,
      project: updates.project ?? existing?.project ?? "",
      label: updates.label ?? existing?.label,
      ignored: updates.ignored ?? existing?.ignored,
    };

    if (!nextMapping.label) {
      delete nextMapping.label;
    }

    if (!nextMapping.ignored) {
      delete nextMapping.ignored;
    }

    const nextMappings = existing
      ? workspaceMappings.map((mapping) =>
          mapping.workspaceKey === workspaceKey ? nextMapping : mapping,
        )
      : [
          ...workspaceMappings,
          {
            ...nextMapping,
            label: nextMapping.label ?? workspace?.workspaceLabel,
          },
        ];

    try {
      await saveWorkspaceMappings(nextMappings);
      workspaceMappings = nextMappings;
      return true;
    } catch (saveError) {
      activityError =
        saveError instanceof Error
          ? saveError.message
          : "Could not save workspace mapping.";
      return false;
    }
  };

  const setTrackingEnabled = async (enabled: boolean) => {
    activityError = "";

    try {
      if (enabled) {
        await startActivityTracking();
      } else {
        await stopActivityTracking();
      }

      trackingStatus = await getTrackingStatus();
    } catch (toggleError) {
      activityError =
        toggleError instanceof Error
          ? toggleError.message
          : "Could not update tracking.";
    }
  };

  const updateTrackingSettings = async (settings: TrackingSettings) => {
    activityError = "";

    try {
      await saveTrackingSettings(settings);
      trackingSettings = settings;
    } catch (saveError) {
      activityError =
        saveError instanceof Error
          ? saveError.message
          : "Could not save tracking settings.";
    }
  };

  const recordSuggestionState = async (
    suggestion: ActivitySuggestion,
    status: SuggestionStateEntry["status"],
  ) => {
    const entry: SuggestionStateEntry = {
      suggestionId: suggestion.id,
      status,
      updatedAt: new Date().toISOString(),
    };

    const nextState = [
      ...suggestionState.filter((item) => item.suggestionId !== suggestion.id),
      entry,
    ];

    await saveSuggestionState(nextState);
    suggestionState = nextState;
  };

  const finalizeSuggestion = async (suggestion: ActivitySuggestion) => {
    const acceptedIds = suggestion.id.endsWith(":consolidated")
      ? [
          suggestion.id,
          ...activitySuggestions
            .filter((item) => item.workspaceKey === suggestion.workspaceKey)
            .map((item) => item.id),
        ]
      : [suggestion.id];

    const acceptedAt = new Date().toISOString();
    const nextState = [
      ...suggestionState.filter(
        (item) => !acceptedIds.includes(item.suggestionId),
      ),
      ...acceptedIds.map((suggestionId) => ({
        suggestionId,
        status: "accepted" as const,
        updatedAt: acceptedAt,
      })),
    ];

    await saveSuggestionState(nextState);
    suggestionState = nextState;
  };

  const startEntryFromSuggestion = (suggestion: ActivitySuggestion) => {
    pendingSuggestion = suggestion;
    selectedDate = suggestion.date;
    project = suggestion.project ?? "";
    startTime = suggestion.startTime;
    endTime = suggestion.endTime;
    notes = suggestion.note;
    editingSlot = null;
    error = "";
    openEntryModal();
  };

  const editSuggestion = (suggestion: ActivitySuggestion) => {
    startEntryFromSuggestion(suggestion);
  };

  const dismissSuggestion = async (suggestion: ActivitySuggestion) => {
    await recordSuggestionState(suggestion, "dismissed");
  };
</script>

<main class="app-shell">
  <div class="screen-viewport">
    <div class="screen-track" data-screen={activeScreen}>
      <DashboardScreen
        {selectedDate}
        {totalForSelectedDay}
        {dayEntries}
        {isLoading}
        onOpenWorkspaces={openWorkspaces}
        onOpenRmSettings={openRmSettings}
        onMoveDay={moveDay}
        onJumpToToday={jumpToToday}
        onAddEntry={openAddEntry}
        onAddSlotAfter={addSlotAfter}
        onEditSlot={editSlot}
        onDeleteSlot={deleteSlot}
        {slotsForDisplay}
      />

      <WorkspacesScreen
        {knownWorkspaces}
        {workspaceMappings}
        {projectSuggestions}
        {trackingStatus}
        {trackingSettings}
        {activitySuggestions}
        {activitySegments}
        {suggestionState}
        {entries}
        {selectedDate}
        onBack={closeWorkspaces}
        onToggleTracking={setTrackingEnabled}
        onSaveMapping={upsertWorkspaceMapping}
        onSaveSettings={updateTrackingSettings}
        onCreateEntry={startEntryFromSuggestion}
        onEditSuggestion={editSuggestion}
        onDismissSuggestion={dismissSuggestion}
      />

      <RmSettingsScreen onBack={closeRmSettings} />
    </div>
  </div>

  {#if activityError}
    <p class="form-error activity-banner" role="alert">{activityError}</p>
  {/if}

  <EntryFormModal
    open={entryModalOpen}
    bind:project
    bind:startTime
    bind:endTime
    bind:notes
    {error}
    {isSaving}
    {isLoading}
    {isEditing}
    {pendingSuggestion}
    {projectSuggestions}
    onSubmit={submitEntry}
    onClose={closeEntryModal}
  />
</main>
