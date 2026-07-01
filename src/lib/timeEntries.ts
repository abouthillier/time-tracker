import { invoke } from '@tauri-apps/api/core'

export type TimeSlot = {
  startTime: string
  endTime: string
  notes?: string
}

export type TimeEntry = {
  id: string
  date: string
  project: string
  entries: TimeSlot[]
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

const timeToMinutes = (time: string) => {
  const [hours, minutes] = time.split(':').map(Number)

  return hours * 60 + minutes
}
