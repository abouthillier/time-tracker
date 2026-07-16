import type { WorkspaceMapping } from './activity'
import type { TimeEntry } from './timeEntries'
import type { RmCatalogCache, RmProjectCatalogEntry } from './rm'

export const isCatalogReady = (catalog: RmCatalogCache | null | undefined) =>
  Boolean(catalog && catalog.projects.length > 0)

export const CATALOG_STALE_DAYS = 7

const MS_PER_DAY = 24 * 60 * 60 * 1000

export const catalogAgeDays = (fetchedAt?: string) => {
  if (!fetchedAt) {
    return null
  }

  const fetched = new Date(fetchedAt)
  if (Number.isNaN(fetched.getTime())) {
    return null
  }

  return Math.floor((Date.now() - fetched.getTime()) / MS_PER_DAY)
}

export const isCatalogStale = (
  fetchedAt?: string,
  maxDays = CATALOG_STALE_DAYS,
) => {
  const ageDays = catalogAgeDays(fetchedAt)
  return ageDays !== null && ageDays > maxDays
}

export const findCatalogProject = (
  catalog: RmCatalogCache | null | undefined,
  assignableId: number | null | undefined,
) =>
  catalog?.projects.find((project) => project.assignableId === assignableId) ??
  null

export const findCatalogProjectByName = (
  catalog: RmCatalogCache | null | undefined,
  name: string,
) => {
  const normalized = name.trim().toLowerCase()
  if (!normalized || !catalog) {
    return null
  }

  return (
    catalog.projects.find(
      (project) => project.name.trim().toLowerCase() === normalized,
    ) ?? null
  )
}

export const categoriesForCatalog = (
  catalog: RmCatalogCache | null | undefined,
) => catalog?.categories ?? []

/** @deprecated Use categoriesForCatalog — categories are account-wide. */
export const categoriesForProject = (
  project: RmProjectCatalogEntry | null,
  catalog?: RmCatalogCache | null,
) => categoriesForCatalog(catalog ?? null)

export const suggestRmProjects = (
  catalog: RmCatalogCache,
  query: string,
  limit = 8,
) => {
  const normalized = query.trim().toLowerCase()
  if (!normalized) {
    return catalog.projects.slice(0, limit)
  }

  const scored = catalog.projects
    .map((project) => ({
      project,
      score: scoreProjectMatch(project, normalized),
    }))
    .filter((item) => item.score > 0)
    .sort((left, right) => right.score - left.score)

  if (scored.length > 0) {
    return scored.slice(0, limit).map((item) => item.project)
  }

  return catalog.projects
    .filter((project) => project.name.toLowerCase().includes(normalized))
    .slice(0, limit)
}

const scoreProjectMatch = (
  project: RmProjectCatalogEntry,
  query: string,
) => {
  const name = project.name.toLowerCase()
  const code = project.projectCode?.toLowerCase() ?? ''

  if (name === query || code === query) {
    return 100
  }

  if (name.startsWith(query) || code.startsWith(query)) {
    return 80
  }

  if (name.includes(query) || code.includes(query)) {
    return 60
  }

  const acronym = acronymFor(name)
  if (acronym === query || acronym.includes(query)) {
    return 50
  }

  const queryWords = query.split(/\s+/).filter(Boolean)
  if (queryWords.every((word) => name.includes(word))) {
    return 40
  }

  return 0
}

const acronymFor = (value: string) =>
  value
    .split(/[^a-zA-Z0-9]+/)
    .filter(Boolean)
    .map((part) => part[0]?.toLowerCase() ?? '')
    .join('')

export const isLegacyEntry = (entry: TimeEntry) =>
  entry.assignableId == null || !entry.category?.trim()

export const isLegacyMapping = (
  mapping: WorkspaceMapping,
  catalog: RmCatalogCache | null,
) => {
  if (!mapping.project?.trim()) {
    return false
  }

  if (mapping.assignableId != null) {
    return false
  }

  if (!catalog) {
    return true
  }

  return !findCatalogProjectByName(catalog, mapping.project)
}

export const collectLegacyProjectKeys = (
  entries: TimeEntry[],
  mappings: WorkspaceMapping[],
  catalog: RmCatalogCache | null,
) => {
  const keys = new Set<string>()

  for (const entry of entries) {
    if (isLegacyEntry(entry) && entry.project.trim()) {
      keys.add(entry.project.trim())
    }
  }

  for (const mapping of mappings) {
    if (isLegacyMapping(mapping, catalog) && mapping.project.trim()) {
      keys.add(mapping.project.trim())
    }
  }

  return [...keys].sort((a, b) => a.localeCompare(b))
}

export type LegacyProjectResolution = {
  assignableId: number
  project: string
  category: string
}

export const applyLegacyResolutions = ({
  entries,
  mappings,
  resolutions,
}: {
  entries: TimeEntry[]
  mappings: WorkspaceMapping[]
  resolutions: Record<string, LegacyProjectResolution>
}) => {
  const nextEntries = entries.map((entry) => {
    const legacyKey = entry.project.trim()
    const resolution = resolutions[legacyKey]

    if (!resolution || !isLegacyEntry(entry)) {
      return entry
    }

    return {
      ...entry,
      assignableId: resolution.assignableId,
      project: resolution.project,
      category: resolution.category,
    }
  })

  const nextMappings = mappings.map((mapping) => {
    const legacyKey = mapping.project.trim()
    const resolution = resolutions[legacyKey]

    if (!resolution || mapping.assignableId != null) {
      return mapping
    }

    return {
      ...mapping,
      assignableId: resolution.assignableId,
      project: resolution.project,
      defaultCategory: resolution.category,
    }
  })

  return { entries: nextEntries, mappings: nextMappings }
}

export const syncEntryLabelsFromCatalog = (
  entries: TimeEntry[],
  catalog: RmCatalogCache | null,
) => {
  if (!catalog) {
    return entries
  }

  return entries.map((entry) => {
    if (entry.assignableId == null) {
      return entry
    }

    const catalogProject = findCatalogProject(catalog, entry.assignableId)
    if (!catalogProject) {
      return entry
    }

    return {
      ...entry,
      project: catalogProject.name,
    }
  })
}
