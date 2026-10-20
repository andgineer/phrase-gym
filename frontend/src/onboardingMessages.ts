import type { Language } from './api'
import { sr, de } from './onboardingEuropeanMessages'

const en = {
  missingFields:
    'Complete the lesson title, phrase, situation, performer and audio locale, then choose or drop the original audio again. Your input is retained.',
  automatic:
    'Complete the text below, then choose, drop or paste the original audio. Delivery starts automatically.',
  retryStorage: 'Retry local storage',
  conflict:
    'Saved delivery changed in another tab. Review the saved lesson before retrying or discarding.',
  accepted: 'Accepted by the Server. Check lesson status for current preparation progress.',
  statusUnavailable:
    'The saved lesson is accepted, but its current status is unavailable to this connection. Check the Server connection and account before retrying.',
  empty: 'No published phrases in this language yet. Choosing a language does not add a lesson.',
  add: 'Add a first lesson',
  supported:
    'Submit one fixed English phrase with its original reference audio. Keep Coach running and connected with the bounded Claude preparation setup. Your exact wording and original audio are preserved.',
  unsupported:
    'New lesson input currently supports English only. Published material in other languages remains available in its library.',
  title: 'Lesson title',
  phrase: 'Exact English phrase',
  context: 'Situation or meaning',
  note: 'Optional note',
  audio: 'Original reference audio (WAV, MP3, OGG or MP4; up to 2 MB)',
  performer: 'Reference performer or source speaker',
  abandon: 'Discard local saved input (accepted work stays on the Server)',
  locale: 'Audio locale (for example en-US)',
  submit: 'Submit lesson',
  close: 'Close',
  refresh: 'Check lesson status',
  retry: 'Retry the same delivery',
  another: 'Add another lesson',
  busy: 'Saving and sending lesson…',
  unsent: 'Delivery is unconfirmed. Your saved input will be retried with the same identity.',
  waiting: 'Accepted — waiting for Coach. You can close this panel; the Server has your lesson.',
  preparing: 'Coach is preparing the accepted lesson.',
  pending_delivery: 'Preparation finished; waiting for confirmed publication.',
  published: 'Published. Open the lesson to see its supported practice options.',
  failed: 'Preparation failed. Check the reason below and fix Coach before retrying preparation.',
  cancelled: 'Preparation was cancelled. Your accepted input remains on the Server.',
  superseded: 'This input was replaced by a newer version.',
  retryPreparation: 'Retry preparation',
  open: 'Open published lesson',
  referenceOnly:
    'Ready for ungraded study. Start a session to read, listen, record and compare. Memory ratings are not part of this study.',
  referenceCount: 'Ready for ungraded study',
  awaiting: 'Awaiting preparation or repair',
  permission:
    'This connection cannot submit material. Ask the Server operator to enable supported intake; your input is retained. No lesson has been confirmed accepted.',
  invalidAudio: 'Select a supported, nonempty audio file of at most 2 MB.',
  storage:
    'Could not save input on this device. No submission was sent. Keep this panel open and restore browser storage before retrying.',
}
export type OnboardingMessages = typeof en
const ru: OnboardingMessages = {
  ...en,
  missingFields:
    'Заполните название урока, фразу, ситуацию, исполнителя и языковой вариант аудио, затем снова выберите или перетащите исходное аудио. Материал сохранён.',
  automatic:
    'Заполните текст ниже, затем выберите, перетащите или вставьте исходное аудио. Отправка начнётся автоматически.',
  conflict:
    'Сохранённая отправка изменилась в другой вкладке. Проверьте сохранённый урок перед повтором или удалением.',
  accepted: 'Принято сервером. Проверьте статус урока, чтобы узнать о подготовке.',
  statusUnavailable:
    'Урок принят, но его текущий статус недоступен этому подключению. Проверьте подключение к серверу и учётную запись перед повтором.',
  retryStorage: 'Проверить хранилище браузера ещё раз',
  performer: 'Исполнитель или исходный диктор',
  abandon: 'Удалить локальный материал (принятые задания остаются на сервере)',
  empty: 'На этом языке пока нет опубликованных фраз. Выбор языка не добавляет урок.',
  add: 'Добавить первый урок',
  supported:
    'Можно отправить одну фиксированную английскую фразу с исходным аудио. Coach должен работать и быть подключён с настроенной ограниченной подготовкой через Claude. Точный текст и исходное аудио сохраняются.',
  unsupported:
    'Добавление уроков пока поддерживает только английский. Опубликованные материалы на других языках доступны в их библиотеке.',
  title: 'Название урока',
  phrase: 'Точная английская фраза',
  context: 'Ситуация или значение',
  note: 'Необязательная заметка',
  audio: 'Исходное аудио (WAV, MP3, OGG или MP4; до 2 МБ)',
  locale: 'Языковой вариант аудио (например en-US)',
  submit: 'Отправить урок',
  close: 'Закрыть',
  refresh: 'Проверить статус урока',
  retry: 'Повторить ту же отправку',
  another: 'Добавить ещё урок',
  busy: 'Сохраняем и отправляем урок…',
  unsent:
    'Доставка не подтверждена. Сохранённый материал будет отправлен с тем же идентификатором.',
  waiting: 'Принято — ожидает Coach. Можно закрыть панель: урок сохранён на сервере.',
  preparing: 'Coach готовит принятый урок.',
  pending_delivery: 'Подготовка завершена; ожидаем подтверждения публикации.',
  published: 'Опубликовано. Откройте урок, чтобы увидеть доступные варианты практики.',
  failed: 'Ошибка подготовки. Проверьте причину ниже и исправьте Coach перед повтором.',
  cancelled: 'Подготовка отменена. Принятый материал остаётся на сервере.',
  superseded: 'Этот материал заменён новой версией.',
  retryPreparation: 'Повторить подготовку',
  open: 'Открыть опубликованный урок',
  referenceOnly:
    'Готово для репетиции по образцу. Откройте фразу: слушайте, записывайте и сравнивайте. Плановая проверка вспоминания для этих материалов пока не поддерживается.',
  referenceCount: 'Готово только для репетиции по образцу',
  awaiting: 'Ожидает подготовки или исправления',
  permission:
    'Это подключение не может отправлять материал. Попросите оператора сервера настроить приём; материал сохранён. Приём урока не подтверждён.',
  invalidAudio: 'Выберите поддерживаемый непустой аудиофайл размером до 2 МБ.',
  storage:
    'Не удалось сохранить материал в браузере. Ничего не отправлено. Оставьте панель открытой и восстановите доступ к хранилищу перед повтором.',
}
export const onboardingMessages = (locale: Language): OnboardingMessages =>
  ({ en, ru, sr, de })[locale]
