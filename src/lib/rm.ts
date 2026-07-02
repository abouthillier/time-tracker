import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export type RmProjectCatalogEntry = {
  assignableId: number
  name: string
  projectCode?: string
  phaseName?: string | null
  parentProjectId?: number | null
  client?: string | null
  categories: string[]
}

export type RmCatalogCache = {
  fetchedAt: string
  categories?: string[]
  projects: RmProjectCatalogEntry[]
}

export type RmLinkedUser = {
  id: number
  email: string
  displayName: string
  linkedAt: string
}

export type RmSettings = {
  region: string
  catalogFetchedAt?: string
  legacyMigrationCompletedAt?: string
  linkedUser?: RmLinkedUser
}

export type RmUserSummary = {
  id: number
  email: string
  displayName: string
}

export type RmUserCandidates = {
  candidates: RmUserSummary[]
}

export type SyncScope = 'changed' | 'failed' | 'selected' | 'all'

export type SyncDayRequest = {
  date: string
  dryRun: boolean
  scope: SyncScope
  entryIds?: string[]
  excludeEntryIds?: string[]
}

export type SyncPreviewRow = {
  entryId: string
  project: string
  category: string
  hours: number
  action: string
  hash: string
  included: boolean
}

export type SyncFailure = {
  entryId: string
  date: string
  project: string
  category: string
  error: string
  recoverable: boolean
}

export type SyncDayResult = {
  succeeded: number
  skipped: number
  failed: SyncFailure[]
  preview?: SyncPreviewRow[]
  linkedUser?: RmLinkedUser
}

export type RmCatalogProgress = {
  current: number
  total: number
  message: string
}

export const saveRmToken = (token: string) =>
  invoke<void>('rm_save_token', { token })

export const clearRmToken = () => invoke<void>('rm_clear_token')

export const hasRmToken = () => invoke<boolean>('rm_has_token')

export const invokeErrorMessage = (error: unknown, fallback: string) => {
  if (typeof error === 'string' && error.trim()) {
    return error
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message
  }

  return fallback
}

export const testRmConnection = (token?: string) =>
  invoke<void>('rm_test_connection', { token: token?.trim() || null })

export const loadRmSettings = () => invoke<RmSettings>('rm_load_settings')

export const saveRmSettings = (settings: RmSettings) =>
  invoke<void>('rm_save_settings', { settings })

export const loadRmCatalog = () =>
  invoke<RmCatalogCache | null>('rm_load_catalog')

export const refreshRmCatalog = (token?: string) =>
  invoke<RmCatalogCache>('rm_refresh_catalog', { token: token?.trim() || null })

export const listenRmCatalogProgress = (
  handler: (progress: RmCatalogProgress) => void,
) =>
  listen<RmCatalogProgress>('rm-catalog-progress', (event) => {
    handler(event.payload)
  })

export const formatCatalogAge = (fetchedAt?: string) => {
  if (!fetchedAt) {
    return 'Never refreshed'
  }

  const fetched = new Date(fetchedAt)
  if (Number.isNaN(fetched.getTime())) {
    return 'Unknown'
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(fetched)
}

export const findRmUserCandidates = (email: string, token?: string) =>
  invoke<RmUserCandidates>('rm_find_user_candidates', {
    email,
    token: token?.trim() || null,
  })

export const linkRmUser = (
  userId: number,
  email: string,
  displayName: string,
) =>
  invoke<RmLinkedUser>('rm_link_user', {
    userId,
    email,
    displayName,
  })

export const unlinkRmUser = () => invoke<void>('rm_unlink_user')

export const getRmLinkedUser = () =>
  invoke<RmLinkedUser | null>('rm_get_linked_user')

export const syncRmDay = (request: SyncDayRequest) =>
  invoke<SyncDayResult>('rm_sync_day', { request })
