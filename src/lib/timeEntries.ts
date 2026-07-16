import { invoke } from '@tauri-apps/api/core'

export type RmEntrySyncStatus = 'pending' | 'synced' | 'dirty' | 'error'

export type RmEntrySync = {
  remoteId?: number
  lastSyncedAt?: string
  syncedHash?: string
  status: RmEntrySyncStatus
  lastError?: string
  lastErrorAt?: string
  lastAttemptAt?: string
}

export type TimeSlot = {
  startTime: string
  endTime: string
  notes?: string
}

export type TimeEntry = {
  id: string
  date: string
  project: string
  assignableId?: number
  category?: string
  entries: TimeSlot[]
  rmSync?: RmEntrySync
}

export type EntryTarget = {
  project: string
  assignableId?: number
  category?: string
}

export type EntryDraft = Omit<TimeEntry, 'id'>

export const todayKey = () => dateToKey(new Date())

export const dateToKey = (date: Date) => {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')

  return `${year}-${month}-${day}`
}

export const shiftDateKey = (dateKey: string, days: number) => {
  const [year, month, day] = dateKey.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  date.setDate(date.getDate() + days)

  return dateToKey(date)
}

const dateFromKey = (dateKey: string) => {
  const [year, month, day] = dateKey.split('-').map(Number)
  return new Date(year, month - 1, day)
}

export const formatWeekdayLabel = (dateKey: string) =>
  new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(
    dateFromKey(dateKey),
  )

export const formatDateSubLabel = (dateKey: string) =>
  new Intl.DateTimeFormat(undefined, {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  }).format(dateFromKey(dateKey))

export const formatDateLabel = (dateKey: string) =>
  new Intl.DateTimeFormat(undefined, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  }).format(dateFromKey(dateKey))

export const minutesBetween = (startTime: string, endTime: string) => {
  const start = timeToMinutes(startTime)
  const end = timeToMinutes(endTime)

  return Math.max(0, end - start)
}

export const formatMinutes = (minutes: number) => {
  const hours = Math.floor(minutes / 60)
  const remainder = minutes % 60

  if (hours === 0) return `${remainder}m`
  if (remainder === 0) return `${hours}h`

  return `${hours}h ${remainder}m`
}

export const slotDuration = (slot: TimeSlot) =>
  minutesBetween(slot.startTime, slot.endTime)

export const entryDuration = (entry: Pick<TimeEntry, 'entries'>) =>
  entry.entries.reduce((total, slot) => total + slotDuration(slot), 0)

const syncNotesFromEntry = (entry: Pick<TimeEntry, 'entries'>) => {
  const notes = entry.entries
    .map((slot) => slot.notes?.trim())
    .filter((note): note is string => Boolean(note))
    .join('; ')

  return [...notes].length > 256 ? [...notes].slice(0, 256).join('') : notes
}

const syncHoursFromEntry = (entry: Pick<TimeEntry, 'entries'>) => {
  const totalMinutes = entryDuration(entry)
  return Math.round((totalMinutes / 60) * 100) / 100
}

export const computeSyncHash = (
  entry: Pick<TimeEntry, 'date' | 'assignableId' | 'category' | 'entries'>,
) => {
  const assignableId = entry.assignableId ?? 0
  const category = entry.category ?? ''
  const hours = syncHoursFromEntry(entry)
  const notes = syncNotesFromEntry(entry)

  return `${entry.date}|${assignableId}|${category}|${hours.toFixed(2)}|${notes}`
}

export const clearRemoteLink = (entry: TimeEntry): TimeEntry => {
  if (!entry.rmSync) {
    return entry
  }

  return {
    ...entry,
    rmSync: {
      ...entry.rmSync,
      remoteId: undefined,
      syncedHash: undefined,
      lastSyncedAt: undefined,
      lastError: undefined,
      lastErrorAt: undefined,
      status: 'pending',
    },
  }
}

export const markEntryDirty = (entry: TimeEntry): TimeEntry => {
  if (!entry.rmSync || entry.rmSync.status !== 'synced') {
    return entry
  }

  if (entry.rmSync.syncedHash === computeSyncHash(entry)) {
    return entry
  }

  return {
    ...entry,
    rmSync: {
      ...entry.rmSync,
      status: 'dirty',
    },
  }
}

export const prepareEntriesForSave = (
  previousEntries: TimeEntry[],
  nextEntries: TimeEntry[],
) =>
  nextEntries.map((entry) => {
    const previous = previousEntries.find((item) => item.id === entry.id)
    if (!previous) {
      return entry
    }

    const identityChanged =
      previous.date !== entry.date ||
      previous.assignableId !== entry.assignableId ||
      (previous.category?.trim() ?? '') !== (entry.category?.trim() ?? '')

    let updated = identityChanged ? clearRemoteLink(entry) : entry
    updated = markEntryDirty(updated)

    return updated
  })

export const isRmReadyEntry = (entry: TimeEntry) =>
  entry.assignableId != null && Boolean(entry.category?.trim())

export const entryRmSyncStatus = (
  entry: TimeEntry,
): RmEntrySyncStatus | null => {
  if (!isRmReadyEntry(entry)) {
    return null
  }

  return entry.rmSync?.status ?? 'pending'
}

export const entrySyncStatusLabel = (status: RmEntrySyncStatus) => {
  switch (status) {
    case 'synced':
      return 'Synced to Resource Management'
    case 'dirty':
      return 'Changed since last sync'
    case 'error':
      return 'Sync failed'
    case 'pending':
      return 'Not synced yet'
  }
}

export const markEntryResolved = (entry: TimeEntry): TimeEntry => {
  const existing = entry.rmSync

  return {
    ...entry,
    rmSync: {
      remoteId: existing?.remoteId,
      lastSyncedAt: existing?.lastSyncedAt,
      syncedHash: existing?.syncedHash ?? computeSyncHash(entry),
      status: 'synced',
      lastAttemptAt: existing?.lastAttemptAt,
    },
  }
}

export const updateEntryById = (
  currentEntries: TimeEntry[],
  entryId: string,
  updater: (entry: TimeEntry) => TimeEntry,
) =>
  currentEntries.map((entry) =>
    entry.id === entryId ? updater(entry) : entry,
  )

export const totalMinutesForDay = (entries: TimeEntry[], dateKey: string) =>
  entries
    .filter((entry) => entry.date === dateKey)
    .reduce((total, entry) => total + entryDuration(entry), 0)

export const earliestEntryStart = (entry: Pick<TimeEntry, 'entries'>) =>
  entry.entries.reduce<string | null>(
    (earliestStartTime, slot) =>
      earliestStartTime === null || slot.startTime < earliestStartTime
        ? slot.startTime
        : earliestStartTime,
    null,
  )

export const latestEntryEnd = (entry: Pick<TimeEntry, 'entries'>) =>
  entry.entries.reduce<string | null>(
    (latestEndTime, slot) =>
      latestEndTime === null || slot.endTime > latestEndTime
        ? slot.endTime
        : latestEndTime,
    null,
  )

export const loadEntries = () => invoke<TimeEntry[]>('load_entries')

export const saveEntries = (entries: TimeEntry[]) =>
  invoke<void>('save_entries', { entries })

export const normalizeProject = (projectName: string) =>
  projectName.trim().toLocaleLowerCase()

export const isSameProject = (firstProject: string, secondProject: string) =>
  normalizeProject(firstProject) === normalizeProject(secondProject)

export const entryTargetKey = (target: EntryTarget) => {
  if (target.assignableId != null) {
    return `${target.assignableId}::${target.category?.trim() ?? ''}`
  }

  return normalizeProject(target.project)
}

export const entryMatchesTarget = (
  entry: TimeEntry,
  date: string,
  target: EntryTarget,
) => {
  if (entry.date !== date) {
    return false
  }

  if (entry.assignableId != null && target.assignableId != null) {
    return (
      entry.assignableId === target.assignableId &&
      (entry.category?.trim() ?? '') === (target.category?.trim() ?? '')
    )
  }

  return isSameProject(entry.project, target.project)
}

const sortSlots = (slots: TimeSlot[]) =>
  [...slots].sort(
    (left, right) =>
      left.startTime.localeCompare(right.startTime) ||
      left.endTime.localeCompare(right.endTime),
  )

export const upsertSlot = (
  currentEntries: TimeEntry[],
  date: string,
  target: EntryTarget,
  slot: TimeSlot,
) => {
  let didAppendSlot = false

  const nextEntries = currentEntries.map((entry) => {
    if (entryMatchesTarget(entry, date, target)) {
      didAppendSlot = true

      return {
        ...entry,
        project: target.project,
        assignableId: target.assignableId,
        category: target.category,
        entries: sortSlots([...entry.entries, slot]),
      }
    }

    return entry
  })

  if (didAppendSlot) {
    return nextEntries
  }

  return [
    ...nextEntries,
    {
      id: crypto.randomUUID(),
      date,
      project: target.project,
      assignableId: target.assignableId,
      category: target.category,
      entries: [slot],
    },
  ]
}

export const moveEditedSlot = (
  currentEntries: TimeEntry[],
  slotToEdit: { entryId: string; slotIndex: number },
  date: string,
  target: EntryTarget,
  slot: TimeSlot,
) => {
  const originalEntry = currentEntries.find(
    (entry) => entry.id === slotToEdit.entryId,
  )

  if (
    originalEntry &&
    entryMatchesTarget(originalEntry, date, target)
  ) {
    return currentEntries.map((entry) =>
      entry.id === slotToEdit.entryId
        ? {
            ...entry,
            project: target.project,
            assignableId: target.assignableId,
            category: target.category,
            entries: sortSlots(
              entry.entries.map((existingSlot, index) =>
                index === slotToEdit.slotIndex ? slot : existingSlot,
              ),
            ),
          }
        : entry,
    )
  }

  return upsertSlot(
    removeSlot(currentEntries, slotToEdit.entryId, slotToEdit.slotIndex),
    date,
    target,
    slot,
  )
}

export const removeSlot = (
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
    .filter((entry) => entry.entries.length > 0)

const timeToMinutes = (time: string) => {
  const [hours, minutes] = time.split(':').map(Number)

  return hours * 60 + minutes
}

const minutesToTime = (totalMinutes: number) => {
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60

  return `${`${hours}`.padStart(2, '0')}:${`${minutes}`.padStart(2, '0')}`
}

/** Display "09:45" as "9:45am". */
export const formatClockTime = (time: string) => {
  const totalMinutes = timeToMinutes(time)
  const hours24 = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  const period = hours24 >= 12 ? 'pm' : 'am'
  const hours12 = hours24 % 12 || 12

  return `${hours12}:${`${minutes}`.padStart(2, '0')}${period}`
}

export type TimeGap = {
  startTime: string
  endTime: string
}

/**
 * Gaps between covered intervals from the earliest slot start to the
 * latest slot end on the given entries (overlapping slots are merged).
 */
export const dayEntryGaps = (entries: TimeEntry[]): TimeGap[] => {
  const intervals = entries
    .flatMap((entry) => entry.entries)
    .map((slot) => ({
      start: timeToMinutes(slot.startTime),
      end: timeToMinutes(slot.endTime),
    }))
    .filter((interval) => interval.end > interval.start)
    .sort((left, right) => left.start - right.start || left.end - right.end)

  if (intervals.length < 2) {
    return []
  }

  const merged: { start: number; end: number }[] = []

  for (const interval of intervals) {
    const last = merged[merged.length - 1]

    if (!last || interval.start > last.end) {
      merged.push({ ...interval })
      continue
    }

    if (interval.end > last.end) {
      last.end = interval.end
    }
  }

  const gaps: TimeGap[] = []

  for (let index = 1; index < merged.length; index += 1) {
    const previous = merged[index - 1]
    const next = merged[index]

    if (next.start > previous.end) {
      gaps.push({
        startTime: minutesToTime(previous.end),
        endTime: minutesToTime(next.start),
      })
    }
  }

  return gaps
}

export const formatTimeGap = (gap: TimeGap) =>
  `${formatClockTime(gap.startTime)}-${formatClockTime(gap.endTime)}`
