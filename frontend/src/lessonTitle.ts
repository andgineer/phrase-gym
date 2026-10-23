// Mirrors Coach's lesson naming so both inbox senders title a lesson from its items.
const limit = 60
const separator = '·'

export function lessonTitle(items: string[]): string {
  const full = items.join(` ${separator} `)
  if ([...full].length <= limit) return full
  const words = full.split(' ')
  let kept = 0
  let length = 0
  for (const word of words) {
    const next = length + (kept > 0 ? 1 : 0) + [...word].length
    if (next + 4 > limit) break
    length = next
    kept += 1
  }
  if (kept === 0) return `${[...full].slice(0, limit - 1).join('')}…`
  const title = words.slice(0, kept)
  const boundary = title.at(-1) === separator || words[kept] === separator
  if (title.at(-1) === separator) title.pop()
  return boundary ? `${title.join(' ')} ${separator} …` : `${title.join(' ')} …`
}
