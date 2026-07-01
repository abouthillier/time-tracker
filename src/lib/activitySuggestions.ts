import {
  type ActivitySegment,
  type SuggestionStateEntry,
  type WorkspaceMapping,
  editorLabel,
} from './activity'
import {
  type TimeEntry,
  formatMinutes,
  minutesBetween,
  type TimeSlot,
} from './timeEntries'

export type ActivitySuggestion = {
  id: string
  date: string
  editor: ActivitySegment['editor']
  workspaceKey: string
  workspaceLabel: string
  project: string | null
  startTime: string
  endTime: string
  durationMinutes: number
  overlapsExisting: boolean
  ignored: boolean
  note: string
}

const MIN_SUGGESTION_MINUTES = 5
const MERGE_GAP_MINUTES = 2

export type TrackedActivityBlock = {
  id: string
  date: string
  editor: ActivitySegment['editor']
  workspaceKey: string
  workspaceLabel: string
  startTime: string
  endTime: string | null
  durationMinutes: number
  isActive: boolean
}

type MergedBlock = {
  workspaceKey: string
  workspaceLabel: string
  editor: ActivitySegment['editor']
  date: string
  start: Date
  end: Date
  segmentIds: string[]
}

export type WorkspaceActivityGroup = {
  workspaceKey: string
  workspaceLabel: string
  project: string | null
  editor: ActivitySegment['editor']
  totalMinutes: number
  blocks: TrackedActivityBlock[]
  suggestions: ActivitySuggestion[]
  consolidatedSuggestion: ActivitySuggestion | null
}

export const buildWorkspaceActivityGroups = ({
  segments,
  mappings,
  suggestionState,
  entries,
  dateKey,
}: {
  segments: ActivitySegment[]
  mappings: WorkspaceMapping[]
  suggestionState: SuggestionStateEntry[]
  entries: TimeEntry[]
  dateKey: string
}): WorkspaceActivityGroup[] => {
  const blocks = buildTrackedActivity({ segments, mappings, dateKey })
  const suggestions = buildActivitySuggestions({
    segments,
    mappings,
    suggestionState,
    entries,
    dateKey,
  })

  const workspaceKeys = [
    ...new Set([
      ...blocks.map((block) => block.workspaceKey),
      ...suggestions.map((suggestion) => suggestion.workspaceKey),
    ]),
  ]

  return workspaceKeys
    .map((workspaceKey) => {
      const workspaceBlocks = blocks.filter(
        (block) => block.workspaceKey === workspaceKey,
      )
      const workspaceSuggestions = suggestions.filter(
        (suggestion) => suggestion.workspaceKey === workspaceKey,
      )
      const mapping = mappings.find((item) => item.workspaceKey === workspaceKey)
      const workspaceLabel =
        mapping?.label?.trim() ||
        workspaceBlocks[0]?.workspaceLabel ||
        workspaceSuggestions[0]?.workspaceLabel ||
        workspaceKey
      const editor =
        workspaceBlocks[0]?.editor ??
        workspaceSuggestions[0]?.editor ??
        'cursor'
      const totalMinutes = workspaceBlocks.reduce(
        (total, block) => total + block.durationMinutes,
        0,
      )

      return {
        workspaceKey,
        workspaceLabel,
        project: mapping?.project?.trim() || null,
        editor,
        totalMinutes,
        blocks: workspaceBlocks,
        suggestions: workspaceSuggestions,
        consolidatedSuggestion: buildConsolidatedSuggestion({
          blocks: workspaceBlocks,
          mapping,
          editor,
          dateKey,
          entries,
        }),
      }
    })
    .sort((left, right) => left.workspaceLabel.localeCompare(right.workspaceLabel))
}

const buildConsolidatedSuggestion = ({
  blocks,
  mapping,
  editor,
  dateKey,
  entries,
}: {
  blocks: TrackedActivityBlock[]
  mapping?: WorkspaceMapping
  editor: ActivitySegment['editor']
  dateKey: string
  entries: TimeEntry[]
}): ActivitySuggestion | null => {
  const closedBlocks = blocks.filter((block) => block.endTime !== null)
  if (closedBlocks.length === 0) {
    return null
  }

  const project = mapping?.project?.trim() || null
  const workspaceLabel = mapping?.label?.trim() || closedBlocks[0].workspaceLabel
  const startTime = closedBlocks.reduce(
    (earliest, block) =>
      earliest === null || block.startTime < earliest ? block.startTime : earliest,
    null as string | null,
  )
  const endTime = closedBlocks.reduce(
    (latest, block) =>
      latest === null || (block.endTime as string) > latest
        ? (block.endTime as string)
        : latest,
    null as string | null,
  )

  if (!startTime || !endTime) {
    return null
  }

  const durationMinutes = minutesBetween(startTime, endTime)
  if (durationMinutes < MIN_SUGGESTION_MINUTES) {
    return null
  }

  const workspaceKey = closedBlocks[0].workspaceKey
  const overlapsExisting = hasOverlap(entries, dateKey, project, startTime, endTime)

  return {
    id: `${dateKey}:${workspaceKey}:consolidated`,
    date: dateKey,
    editor,
    workspaceKey,
    workspaceLabel,
    project,
    startTime,
    endTime,
    durationMinutes,
    overlapsExisting,
    ignored: mapping?.ignored === true,
    note: `Suggested from ${editorLabel(editor)} workspace ${workspaceLabel}`,
  }
}

export const buildTrackedActivity = ({
  segments,
  mappings,
  dateKey,
}: {
  segments: ActivitySegment[]
  mappings: WorkspaceMapping[]
  dateKey: string
}): TrackedActivityBlock[] => {
  const daySegments = segments.filter((segment) => segment.date === dateKey)
  const closedSegments = daySegments
    .filter((segment) => segment.end !== null)
    .map((segment) => ({
      ...segment,
      end: segment.end as string,
    }))
  const openSegments = daySegments.filter((segment) => segment.end === null)

  const mergedClosed = mergeSegments(closedSegments)
  const blocks: TrackedActivityBlock[] = mergedClosed.map((block) => {
    const mapping = mappings.find(
      (item) => item.workspaceKey === block.workspaceKey,
    )
    const startTime = toTimeString(block.start)
    const endTime = toTimeString(block.end)

    return {
      id: createSuggestionId(block),
      date: block.date,
      editor: block.editor,
      workspaceKey: block.workspaceKey,
      workspaceLabel: mapping?.label?.trim() || block.workspaceLabel,
      startTime,
      endTime,
      durationMinutes: minutesBetween(startTime, endTime),
      isActive: false,
    }
  })

  for (const segment of openSegments) {
    const mapping = mappings.find(
      (item) => item.workspaceKey === segment.workspaceKey,
    )
    const start = new Date(segment.start)
    const end = new Date()
    const startTime = toTimeString(start)
    const endTime = toTimeString(end)

    blocks.push({
      id: segment.id,
      date: segment.date,
      editor: segment.editor,
      workspaceKey: segment.workspaceKey,
      workspaceLabel: mapping?.label?.trim() || segment.workspaceLabel,
      startTime,
      endTime: null,
      durationMinutes: Math.max(
        1,
        Math.round((end.getTime() - start.getTime()) / 60_000),
      ),
      isActive: true,
    })
  }

  return blocks.sort((left, right) =>
    left.startTime.localeCompare(right.startTime),
  )
}

export const buildActivitySuggestions = ({
  segments,
  mappings,
  suggestionState,
  entries,
  dateKey,
}: {
  segments: ActivitySegment[]
  mappings: WorkspaceMapping[]
  suggestionState: SuggestionStateEntry[]
  entries: TimeEntry[]
  dateKey: string
}): ActivitySuggestion[] => {
  const closedSegments = segments
    .filter((segment) => segment.date === dateKey && segment.end !== null)
    .map((segment) => ({
      ...segment,
      end: segment.end as string,
    }))

  const mergedBlocks = mergeSegments(closedSegments)
  const dismissedOrAccepted = new Set(
    suggestionState.map((entry) => entry.suggestionId),
  )

  return mergedBlocks
    .map((block) => {
      const suggestionId = createSuggestionId(block)
      const mapping = mappings.find(
        (item) => item.workspaceKey === block.workspaceKey,
      )
      const ignored = mapping?.ignored === true
      const startTime = toTimeString(block.start)
      const endTime = toTimeString(block.end)
      const durationMinutes = minutesBetween(startTime, endTime)
      const project = mapping?.project?.trim() || null
      const overlapsExisting = hasOverlap(entries, dateKey, project, startTime, endTime)

      return {
        id: suggestionId,
        date: block.date,
        editor: block.editor,
        workspaceKey: block.workspaceKey,
        workspaceLabel: mapping?.label?.trim() || block.workspaceLabel,
        project,
        startTime,
        endTime,
        durationMinutes,
        overlapsExisting,
        ignored,
        note: `Suggested from ${editorLabel(block.editor)} workspace ${mapping?.label?.trim() || block.workspaceLabel}`,
        segmentIds: block.segmentIds,
      }
    })
    .filter(
      (suggestion) =>
        !dismissedOrAccepted.has(suggestion.id) &&
        !suggestion.ignored &&
        suggestion.durationMinutes >= MIN_SUGGESTION_MINUTES,
    )
    .sort((left, right) => left.startTime.localeCompare(right.startTime))
}

export const formatSuggestionSummary = (suggestion: ActivitySuggestion) => {
  const projectLabel = suggestion.project ?? 'Unmapped project'
  return `${editorLabel(suggestion.editor)} · ${suggestion.workspaceLabel} — ${formatMinutes(suggestion.durationMinutes)} (${suggestion.startTime}–${suggestion.endTime}) → ${projectLabel}`
}

const mergeSegments = (
  segments: Array<ActivitySegment & { end: string }>,
): MergedBlock[] => {
  const sorted = [...segments].sort((left, right) =>
    left.start.localeCompare(right.start),
  )

  const merged: MergedBlock[] = []

  for (const segment of sorted) {
    const start = new Date(segment.start)
    const end = new Date(segment.end)
    const last = merged[merged.length - 1]

    if (
      last &&
      last.workspaceKey === segment.workspaceKey &&
      last.editor === segment.editor &&
      gapMinutes(last.end, start) <= MERGE_GAP_MINUTES
    ) {
      if (end > last.end) {
        last.end = end
      }
      last.segmentIds.push(segment.id)
      continue
    }

    merged.push({
      workspaceKey: segment.workspaceKey,
      workspaceLabel: segment.workspaceLabel,
      editor: segment.editor,
      date: segment.date,
      start,
      end,
      segmentIds: [segment.id],
    })
  }

  return merged
}

const createSuggestionId = (block: MergedBlock) =>
  `${block.date}:${block.workspaceKey}:${toTimeString(block.start)}:${toTimeString(block.end)}`

const gapMinutes = (left: Date, right: Date) =>
  Math.max(0, Math.round((right.getTime() - left.getTime()) / 60_000))

const toTimeString = (date: Date) => {
  const hours = `${date.getHours()}`.padStart(2, '0')
  const minutes = `${date.getMinutes()}`.padStart(2, '0')
  return `${hours}:${minutes}`
}

const hasOverlap = (
  entries: TimeEntry[],
  dateKey: string,
  project: string | null,
  startTime: string,
  endTime: string,
) => {
  if (!project) {
    return false
  }

  const normalizedProject = project.trim().toLowerCase()
  const dayEntries = entries.filter(
    (entry) =>
      entry.date === dateKey &&
      entry.project.trim().toLowerCase() === normalizedProject,
  )

  return dayEntries.some((entry) =>
    entry.entries.some((slot) => slotsOverlap(slot, startTime, endTime)),
  )
}

const slotsOverlap = (slot: TimeSlot, startTime: string, endTime: string) => {
  const slotStart = minutesBetween('00:00', slot.startTime)
  const slotEnd = minutesBetween('00:00', slot.endTime)
  const nextStart = minutesBetween('00:00', startTime)
  const nextEnd = minutesBetween('00:00', endTime)

  return slotStart < nextEnd && nextStart < slotEnd
}

export const suggestionDurationLabel = (suggestion: ActivitySuggestion) =>
  formatMinutes(suggestion.durationMinutes)
