import type { components } from './api.generated'

export type LearningLanguage = components['schemas']['OpenDaily']['language']

export const learningLanguages: { code: LearningLanguage; name: string }[] = [
  { code: 'en', name: 'English' },
  { code: 'de', name: 'German' },
]

export const languageName = (code: LearningLanguage) =>
  learningLanguages.find((item) => item.code === code)?.name ?? code

export const isLearningLanguage = (code: string): code is LearningLanguage =>
  learningLanguages.some((item) => item.code === code)
