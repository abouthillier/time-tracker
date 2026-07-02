import { invoke } from '@tauri-apps/api/core'

export type EditorKind = 'cursor' | 'code' | 'vscodium'

export type ActivitySegment = {
  id: string
  date: string
  editor: EditorKind
  workspaceKey: string
  workspaceLabel: string
  start: string
  end: string | null
}

export type WorkspaceMapping = {
  workspaceKey: string
  project: string
  assignableId?: number
  defaultCategory?: string
  label?: string
  ignored?: boolean
}

export type KnownWorkspace = {
  workspaceKey: string
  workspaceLabel: string
  editor?: EditorKind
}

export type TrackingStatus = {
  enabled: boolean
  lastError?: string | null
  openSegmentId?: string | null
}

export type TrackingSettings = {
  enabledEditors: EditorKind[]
  pollIntervalSecs: number
}

export type SuggestionStatus = 'accepted' | 'dismissed'

export type SuggestionStateEntry = {
  suggestionId: string
  status: SuggestionStatus
  updatedAt: string
}

export const loadActivitySegments = () =>
  invoke<ActivitySegment[]>('load_activity_segments')

export const loadWorkspaceMappings = () =>
  invoke<WorkspaceMapping[]>('load_workspace_mappings')

export const saveWorkspaceMappings = (mappings: WorkspaceMapping[]) =>
  invoke<void>('save_workspace_mappings', { mappings })

export const loadSuggestionState = () =>
  invoke<SuggestionStateEntry[]>('load_suggestion_state')

export const saveSuggestionState = (state: SuggestionStateEntry[]) =>
  invoke<void>('save_suggestion_state', { state })

export const loadTrackingSettings = () =>
  invoke<TrackingSettings>('load_tracking_settings')

export const saveTrackingSettings = (settings: TrackingSettings) =>
  invoke<void>('save_tracking_settings', { settings })

export const getTrackingStatus = () => invoke<TrackingStatus>('get_tracking_status')

export const startActivityTracking = () => invoke<void>('start_activity_tracking')

export const stopActivityTracking = () => invoke<void>('stop_activity_tracking')

export const listKnownWorkspaces = () =>
  invoke<KnownWorkspace[]>('list_known_workspaces_command')

export const editorLabel = (editor: EditorKind) => {
  switch (editor) {
    case 'cursor':
      return 'Cursor'
    case 'code':
      return 'VS Code'
    case 'vscodium':
      return 'VSCodium'
  }
}
