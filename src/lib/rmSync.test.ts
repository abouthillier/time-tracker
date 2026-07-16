import { describe, expect, it } from 'vitest'
import {
  clearRemoteLink,
  computeSyncHash,
  entryRmSyncStatus,
  isRmReadyEntry,
  markEntryDirty,
  markEntryResolved,
  type TimeEntry,
} from './timeEntries'
import { catalogAgeDays, isCatalogStale } from './rmCatalog'

const sampleEntry = (overrides: Partial<TimeEntry> = {}): TimeEntry => ({
  id: 'entry-1',
  date: '2026-07-01',
  project: 'ELC 2025',
  assignableId: 42,
  category: 'Admin',
  entries: [
    { startTime: '09:00', endTime: '11:30', notes: 'notes' },
  ],
  ...overrides,
})

describe('computeSyncHash', () => {
  it('matches the Rust sync payload hash format', () => {
    expect(computeSyncHash(sampleEntry())).toBe(
      '2026-07-01|42|Admin|2.50|notes',
    )
  })
})

describe('markEntryDirty', () => {
  it('marks synced entries dirty when slot hours change', () => {
    const synced = sampleEntry({
      rmSync: {
        status: 'synced',
        syncedHash: computeSyncHash(sampleEntry()),
        remoteId: 99,
      },
    })

    const edited = sampleEntry({
      entries: [{ startTime: '09:00', endTime: '12:30' }],
      rmSync: synced.rmSync,
    })

    expect(markEntryDirty(edited).rmSync?.status).toBe('dirty')
  })
})

describe('clearRemoteLink', () => {
  it('resets remote link state to pending', () => {
    const linked = sampleEntry({
      rmSync: {
        status: 'synced',
        remoteId: 99,
        syncedHash: 'hash',
        lastSyncedAt: '2026-07-01T00:00:00Z',
      },
    })

    const cleared = clearRemoteLink(linked)
    expect(cleared.rmSync?.status).toBe('pending')
    expect(cleared.rmSync?.remoteId).toBeUndefined()
    expect(cleared.rmSync?.syncedHash).toBeUndefined()
  })
})

describe('markEntryResolved', () => {
  it('clears sync errors and marks the row synced', () => {
    const errored = sampleEntry({
      rmSync: {
        status: 'error',
        lastError: 'Category missing',
        lastErrorAt: '2026-07-01T00:00:00Z',
      },
    })

    const resolved = markEntryResolved(errored)
    expect(resolved.rmSync?.status).toBe('synced')
    expect(resolved.rmSync?.lastError).toBeUndefined()
  })
})

describe('entryRmSyncStatus', () => {
  it('returns null for legacy entries without RM metadata', () => {
    expect(entryRmSyncStatus(sampleEntry({ assignableId: undefined }))).toBeNull()
  })

  it('defaults RM-ready entries to pending', () => {
    expect(entryRmSyncStatus(sampleEntry())).toBe('pending')
  })
})

describe('isRmReadyEntry', () => {
  it('requires assignableId and category', () => {
    expect(isRmReadyEntry(sampleEntry())).toBe(true)
    expect(isRmReadyEntry(sampleEntry({ category: '' }))).toBe(false)
  })
})

describe('catalog staleness', () => {
  it('flags catalogs older than seven days', () => {
    const staleDate = new Date(Date.now() - 8 * 24 * 60 * 60 * 1000).toISOString()
    expect(isCatalogStale(staleDate)).toBe(true)
    expect(catalogAgeDays(staleDate)).toBeGreaterThanOrEqual(8)
  })

  it('accepts fresh catalogs', () => {
    const freshDate = new Date().toISOString()
    expect(isCatalogStale(freshDate)).toBe(false)
  })
})
