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
  projects: RmProjectCatalogEntry[]
}

export type RmSettings = {
  region: string
  catalogFetchedAt?: string
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
