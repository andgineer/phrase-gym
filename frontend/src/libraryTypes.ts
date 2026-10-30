import type { components } from './api.generated'

export type LessonChoice = components['schemas']['LessonChoice']
export type LessonPage = components['schemas']['LessonPage']
export type PhraseSummary = components['schemas']['PhraseSummary']
export type PhrasePage = components['schemas']['PhrasePage']
export type PhraseDetails = components['schemas']['PhraseDetails']
export type CleanupStatus = components['schemas']['CleanupStatus']
export type DeletionResult = components['schemas']['DeletionResult']
export type DeletionTarget = { id: string; text: string }
