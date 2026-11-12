export type Segment = { text: string; bold: boolean }

// Help text may bold the target with **...**; anything unbalanced stays literal text.
export function emphasisSegments(text: string): Segment[] {
  const parts = text.split('**')
  if (parts.length % 2 === 0) return [{ text, bold: false }]
  return parts.map((part, index) => ({ text: part, bold: index % 2 === 1 })).filter((s) => s.text)
}
