import { describe, expect, it } from 'vitest'
import {
  dayEntryGaps,
  formatClockTime,
  formatTimeGap,
  type TimeEntry,
} from './timeEntries'

const entry = (
  id: string,
  slots: { startTime: string; endTime: string }[],
): TimeEntry => ({
  id,
  date: '2026-07-13',
  project: 'Project',
  entries: slots,
})

describe('formatClockTime', () => {
  it('formats morning and afternoon times', () => {
    expect(formatClockTime('09:45')).toBe('9:45am')
    expect(formatClockTime('10:00')).toBe('10:00am')
    expect(formatClockTime('00:05')).toBe('12:05am')
    expect(formatClockTime('12:00')).toBe('12:00pm')
    expect(formatClockTime('15:30')).toBe('3:30pm')
  })
})

describe('dayEntryGaps', () => {
  it('returns empty when fewer than two intervals', () => {
    expect(dayEntryGaps([])).toEqual([])
    expect(
      dayEntryGaps([entry('a', [{ startTime: '09:00', endTime: '10:00' }])]),
    ).toEqual([])
  })

  it('finds a gap between non-overlapping slots', () => {
    expect(
      dayEntryGaps([
        entry('a', [
          { startTime: '09:00', endTime: '09:45' },
          { startTime: '10:00', endTime: '10:15' },
        ]),
      ]),
    ).toEqual([{ startTime: '09:45', endTime: '10:00' }])
  })

  it('formats gap labels like the banner copy', () => {
    const [gap] = dayEntryGaps([
      entry('a', [
        { startTime: '09:00', endTime: '09:45' },
        { startTime: '10:00', endTime: '10:15' },
      ]),
    ])

    expect(formatTimeGap(gap)).toBe('9:45am-10:00am')
  })

  it('ignores touching intervals and merges overlaps across entries', () => {
    expect(
      dayEntryGaps([
        entry('a', [{ startTime: '09:00', endTime: '09:45' }]),
        entry('b', [{ startTime: '09:45', endTime: '10:15' }]),
      ]),
    ).toEqual([])

    expect(
      dayEntryGaps([
        entry('a', [{ startTime: '09:00', endTime: '10:00' }]),
        entry('b', [{ startTime: '09:30', endTime: '10:30' }]),
        entry('c', [{ startTime: '11:00', endTime: '11:15' }]),
      ]),
    ).toEqual([{ startTime: '10:30', endTime: '11:00' }])
  })

  it('reports multiple gaps between earliest start and latest end', () => {
    expect(
      dayEntryGaps([
        entry('a', [
          { startTime: '08:00', endTime: '08:30' },
          { startTime: '09:00', endTime: '09:15' },
          { startTime: '10:00', endTime: '10:30' },
        ]),
      ]),
    ).toEqual([
      { startTime: '08:30', endTime: '09:00' },
      { startTime: '09:15', endTime: '10:00' },
    ])
  })
})
