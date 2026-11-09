import type { components } from './api.generated'

export type Recording = components['schemas']['SavedRecording']
export type Card = components['schemas']['PracticeCard']
export type Rating = keyof components['schemas']['RatingReturnDays']
export type PracticeState = components['schemas']['PracticeState']
export type Progress = components['schemas']['PracticeProgress']
export type CompletionResult = components['schemas']['CompletionResult']
export type Outcome = Exclude<components['schemas']['CompletionRequest']['outcome'], 'studied'>
export type RecordingPage = components['schemas']['RecordingPage']
export type ExtraResult = components['schemas']['ExtraResult']
