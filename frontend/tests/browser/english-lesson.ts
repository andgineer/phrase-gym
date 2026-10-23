import { randomUUID } from 'node:crypto'
import { expect, type APIRequestContext } from '@playwright/test'
import { tone } from './seed-library'
import type { components } from '../rust-contract/api.generated'

export async function englishLesson(request: APIRequestContext, details = true) {
  const media = await request.post('/api/v1/media', {
    data: {
      mime_type: 'audio/wav',
      audio_base64: tone(),
      metadata: {
        language: 'en',
        text: "Let's get started.",
        role: 'reference',
        origin: 'course',
        source: 'synthetic-preview',
        performer: 'Synthetic',
        locale: 'en-US',
        preparation_version: 'fixture-v1',
      },
    },
  })
  expect(media.ok()).toBe(true)
  const asset = await media.json()
  const accepted = await request.post('/api/v1/inbox/lessons', {
    data: {
      submission_id: randomUUID(),
      item_id: 'meeting',
      expected_generation: null,
      language: 'en',
      title: 'Opening a meeting',
      context: 'All participants are present.',
      note: '',
      phrases: [
        {
          source_id: 'open',
          text: "Let's get started.",
          reference_audio_id: asset.id,
          audio_sha256: asset.sha256,
          expected_revision: null,
        },
      ],
    },
  })
  expect(accepted.ok()).toBe(true)
  const claim = await (
    await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
  ).json()
  await stageEnglishClaim(request, claim, details)
  const publication = await request.post('/api/v1/preparation/publish', { data: claim.fence })
  expect(publication.ok()).toBe(true)
  return { publication: await publication.json(), asset }
}

export async function stageEnglishClaim(
  request: APIRequestContext,
  claim: components['schemas']['JobClaim'],
  details = true,
) {
  let reference = claim.input.phrases[0].reference_audio_id
  if (!reference) {
    const uploaded = await request.post('/api/v1/media', {
      data: {
        mime_type: 'audio/wav',
        audio_base64: tone(),
        metadata: {
          language: 'en',
          text: claim.input.phrases[0].text,
          role: 'reference',
          origin: 'generated',
          source: 'synthetic-test',
          engine: 'fake',
          voice: 'synthetic',
          locale: 'en-US',
          speech_text: claim.input.phrases[0].text,
          preparation_version: 'fixture-v1',
        },
      },
    })
    expect(uploaded.ok(), await uploaded.text()).toBe(true)
    reference = (await uploaded.json()).id
  }
  const staged = await request.post('/api/v1/preparation/stage', {
    data: {
      fence: claim.fence,
      publication_id: randomUUID(),
      adapter: 'synthetic-fake-worker-v1',
      package: {
        schema_version: 'english_phrases_v1',
        namespace: claim.receipt.namespace,
        source_revision: String(claim.receipt.generation),
        language: 'en',
        profile_revision: claim.receipt.profile_revision,
        guide_version: claim.receipt.guide_version,
        assessment_version: 'situation-v1',
        lesson: {
          source_id: claim.input.item_id,
          title: claim.input.title,
          target_source_ids: [claim.input.phrases[0].source_id],
        },
        phrases: [
          {
            source_id: claim.input.phrases[0].source_id,
            text: "Let's get started.",
            meaning_key: 'begin-activity',
            intention_key: 'open-meeting',
            accepted_variants: [],
            situations: [
              {
                id: 'call',
                text: 'Everyone has arrived. Open the meeting.',
                language: 'en',
                audio_id: null,
                partner: null,
                exposed_target_source_ids: [],
              },
            ],
            hint: 'started',
            help: {
              language: 'en',
              meaning: 'Begin a shared activity.',
              usage: null,
            },
            details: details
              ? {
                  language: 'ru',
                  meaning: 'Так предлагают начать общее дело, когда все собрались.',
                  translation: 'Давайте начнём.',
                  usage: 'Нейтрально; так говорит ведущий в начале встречи или урока.',
                  forms: null,
                }
              : null,
            familiarity: { kind: 'new' },
            reference_audio_id: reference,
            expected_revision: null,
          },
        ],
      },
    },
  })
  expect(staged.ok(), await staged.text()).toBe(true)
}
