import { watch } from 'vue'
import { takeTransfers } from '../takeTransfers'
import type { Recording, RecordingPage } from '../studyTypes'
import { usePagedList } from './usePagedList'

export function useSavedRecordings(phraseId: () => string) {
  const takes = usePagedList<Recording, RecordingPage>(
    () => `/phrases/${encodeURIComponent(phraseId())}/recordings`,
    10,
  )
  watch(takeTransfers.uploadingTakes, () => void takes.refresh())
  watch(phraseId, () => void takes.reload(), { immediate: true })
  return takes
}
