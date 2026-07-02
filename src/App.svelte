<script lang="ts">
  import {
    earliestEntryStart,
    isRmReadyEntry,
    latestEntryEnd,
    loadEntries,
    minutesBetween,
    moveEditedSlot,
    prepareEntriesForSave,
    removeSlot,
    saveEntries,
    shiftDateKey,
    todayKey,
    totalMinutesForDay,
    upsertSlot,
    isSameProject,
    type EntryTarget,
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

  import {
    getRmLinkedUser,
    hasRmToken,
    loadRmCatalog,
    loadRmSettings,
    saveRmSettings,
    type RmCatalogCache,
    type RmLinkedUser,
    type RmSettings,
  } from "./lib/rm";

  import {
    collectLegacyProjectKeys,
    findCatalogProject,
    isCatalogReady,
    categoriesForCatalog,
    syncEntryLabelsFromCatalog,
  } from "./lib/rmCatalog";

  import DashboardScreen from "./components/DashboardScreen.svelte";
  import EntryFormModal from "./components/EntryFormModal.svelte";
  import RmMigrationModal from "./components/RmMigrationModal.svelte";
  import RmSettingsScreen from "./components/RmSettingsScreen.svelte";
  import RmSyncModal from "./components/RmSyncModal.svelte";
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
  let assignableId: number | null = null;
  let category = "";
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
  let rmCatalog: RmCatalogCache | null = null;
  let rmSettings: RmSettings | null = null;
  let rmHasToken = false;
  let rmLinkedUser: RmLinkedUser | null = null;
  let migrationOpen = false;
  let syncModalOpen = false;
  let legacyKeys: string[] = [];

  let unlistenSegments: (() => void) | null = null;
  let pendingSuggestion: ActivitySuggestion | null = null;

  const DEFAULT_START_TIME = "09:00";
  const DEFAULT_ENTRY_MINUTES = 60;
  const MINUTES_PER_DAY = 24 * 60;

  $: rmPickersActive = isCatalogReady(rmCatalog);

  $: legacyKeyCount = rmPickersActive
    ? collectLegacyProjectKeys(entries, workspaceMappings, rmCatalog).length
    : 0;

  $: dayEntries = entries
    .filter((entry) => entry.date === selectedDate)
    .sort((a, b) =>
      (earliestEntryStart(a) ?? "").localeCompare(earliestEntryStart(b) ?? ""),
    );

  $: totalForSelectedDay = totalMinutesForDay(entries, selectedDate);

  $: canSyncDay =
    rmHasToken &&
    isCatalogReady(rmCatalog) &&
    Boolean(rmLinkedUser) &&
    dayEntries.some(isRmReadyEntry);

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

  const refreshRmData = async () => {
    try {
      const [catalog, settings, tokenSaved, linkedUser] = await Promise.all([
        loadRmCatalog(),
        loadRmSettings(),
        hasRmToken(),
        getRmLinkedUser(),
      ]);

      rmCatalog = catalog;
      rmSettings = settings;
      rmHasToken = tokenSaved;
      rmLinkedUser = linkedUser ?? settings.linkedUser ?? null;

      if (catalog) {
        entries = syncEntryLabelsFromCatalog(entries, catalog);
      }

      maybeOpenMigration();
    } catch {
      // RM not configured yet — keep local-only mode.
    }
  };

  const maybeOpenMigration = () => {
    if (!isCatalogReady(rmCatalog)) {
      return;
    }

    if (rmSettings?.legacyMigrationCompletedAt) {
      return;
    }

    legacyKeys = collectLegacyProjectKeys(
      entries,
      workspaceMappings,
      rmCatalog,
    );
    migrationOpen = legacyKeys.length > 0;
  };

  const openMigrationWizard = () => {
    if (!isCatalogReady(rmCatalog)) {
      return;
    }

    legacyKeys = collectLegacyProjectKeys(
      entries,
      workspaceMappings,
      rmCatalog,
    );
    migrationOpen = legacyKeys.length > 0;
  };

  onMount(() => {
    const setup = async () => {
      try {
        await Promise.all([refreshActivityData(), refreshRmData()]);
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
      maybeOpenMigration();
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
    assignableId = entry.assignableId ?? null;
    category = entry.category ?? "";
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

  const closeRmSettings = async () => {
    activeScreen = "dashboard";
    await refreshRmData();
  };

  const buildEntryTarget = (): EntryTarget | null => {
    const trimmedProject = project.trim();
    const trimmedCategory = category.trim();

    if (rmPickersActive) {
      if (assignableId == null) {
        return null;
      }

      const catalogProject = findCatalogProject(rmCatalog, assignableId);
      if (!catalogProject) {
        return null;
      }

      if (categoriesForCatalog(rmCatalog).length > 0 && !trimmedCategory) {
        return null;
      }

      return {
        assignableId,
        project: catalogProject.name,
        category: trimmedCategory || undefined,
      };
    }

    if (!trimmedProject) {
      return null;
    }

    return { project: trimmedProject };
  };

  const submitEntry = async () => {
    error = "";

    const target = buildEntryTarget();
    if (!target) {
      error = rmPickersActive
        ? "Choose a project and category before saving."
        : "Add a project name before saving.";
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

    const trimmedNotes = notes.trim();
    const slotValues: TimeSlot = {
      startTime,
      endTime,
      notes: trimmedNotes || undefined,
    };

    const nextEntries =
      editingSlot === null
        ? upsertSlot(entries, selectedDate, target, slotValues)
        : moveEditedSlot(
            entries,
            editingSlot,
            selectedDate,
            target,
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
          project: target.project,
          assignableId: target.assignableId,
          defaultCategory: target.category,
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
    assignableId = entry.assignableId ?? null;
    category = entry.category ?? "";
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
    assignableId = null;
    category = "";
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
      const preparedEntries = prepareEntriesForSave(entries, nextEntries);
      await saveEntries(preparedEntries);
      entries = preparedEntries;
    } catch (saveError) {
      error =
        saveError instanceof Error
          ? saveError.message
          : "Could not save entries.";
    } finally {
      isSaving = false;
    }
  };

  const reloadEntriesAfterSync = async () => {
    entries = await loadEntries();
  };

  const openSyncModal = () => {
    syncModalOpen = true;
  };

  const closeSyncModal = async () => {
    syncModalOpen = false;
    await refreshRmData();
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

  const slotsForDisplay = (entry: TimeEntry): SlotForDisplay[] =>
    entry.entries
      .map((slot, index) => ({ slot, index }))
      .sort((a, b) =>
        a.slot.startTime.localeCompare(b.slot.startTime) ||
        a.slot.endTime.localeCompare(b.slot.endTime),
      );

  const addMinutesToTime = (time: string, minutes: number) => {
    const [hours, currentMinutes] = time.split(":").map(Number);
    const totalMinutes =
      (hours * 60 + currentMinutes + minutes) % MINUTES_PER_DAY;

    return `${Math.floor(totalMinutes / 60)}`.padStart(2, "0") +
      `:${totalMinutes % 60}`.padStart(2, "0");
  };

  const completeMigration = async ({
    entries: nextEntries,
    mappings: nextMappings,
  }: {
    entries: TimeEntry[];
    mappings: WorkspaceMapping[];
  }) => {
    try {
      await saveEntries(nextEntries);
      entries = nextEntries;
      await saveWorkspaceMappings(nextMappings);
      workspaceMappings = nextMappings;

      const nextSettings: RmSettings = {
        region: rmSettings?.region ?? "us",
        catalogFetchedAt: rmSettings?.catalogFetchedAt ?? rmCatalog?.fetchedAt,
        legacyMigrationCompletedAt: new Date().toISOString(),
        linkedUser: rmSettings?.linkedUser ?? rmLinkedUser ?? undefined,
      };
      await saveRmSettings(nextSettings);
      rmSettings = nextSettings;
      migrationOpen = false;
    } catch (saveError) {
      activityError =
        saveError instanceof Error
          ? saveError.message
          : "Could not save migration results.";
    }
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
      assignableId: updates.assignableId ?? existing?.assignableId,
      defaultCategory: updates.defaultCategory ?? existing?.defaultCategory,
      label: updates.label ?? existing?.label,
      ignored: updates.ignored ?? existing?.ignored,
    };

    if (!nextMapping.label) {
      delete nextMapping.label;
    }

    if (!nextMapping.ignored) {
      delete nextMapping.ignored;
    }

    if (nextMapping.assignableId == null) {
      delete nextMapping.assignableId;
    }

    if (!nextMapping.defaultCategory) {
      delete nextMapping.defaultCategory;
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
    const mapping = workspaceMappings.find(
      (item) => item.workspaceKey === suggestion.workspaceKey,
    );

    pendingSuggestion = suggestion;
    selectedDate = suggestion.date;
    project = mapping?.project?.trim() || suggestion.project || "";
    assignableId = mapping?.assignableId ?? null;
    category = mapping?.defaultCategory ?? "";
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
        {canSyncDay}
        onOpenWorkspaces={openWorkspaces}
        onOpenRmSettings={openRmSettings}
        onSyncDay={openSyncModal}
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
        catalog={rmCatalog}
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

      <RmSettingsScreen
        onBack={closeRmSettings}
        {legacyKeyCount}
        onOpenMigration={openMigrationWizard}
      />
    </div>
  </div>

  {#if activityError}
    <p class="form-error activity-banner" role="alert">{activityError}</p>
  {/if}

  <EntryFormModal
    open={entryModalOpen}
    bind:project
    bind:assignableId
    bind:category
    bind:startTime
    bind:endTime
    bind:notes
    catalog={rmCatalog}
    {error}
    {isSaving}
    {isLoading}
    {isEditing}
    {pendingSuggestion}
    onSubmit={submitEntry}
    onClose={closeEntryModal}
  />

  <RmMigrationModal
    open={migrationOpen}
    catalog={rmCatalog ?? { fetchedAt: "", projects: [] }}
    {entries}
    mappings={workspaceMappings}
    {legacyKeys}
    onComplete={completeMigration}
    onClose={() => {
      migrationOpen = false;
    }}
  />

  <RmSyncModal
    open={syncModalOpen}
    {selectedDate}
    linkedUser={rmLinkedUser}
    onClose={closeSyncModal}
    onSyncComplete={reloadEntriesAfterSync}
  />
</main>
