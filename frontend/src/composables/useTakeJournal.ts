import { onMounted, onUnmounted } from 'vue'
import { takeTransfers } from '../takeTransfers'

export function useTakeJournal() {
  function reconnect() {
    if (!document.hidden) void takeTransfers.recover()
  }
  function receipts() {
    if (!document.hidden) void takeTransfers.retryAfterReceipts()
  }
  onMounted(() => {
    void takeTransfers.recover()
    window.addEventListener('online', reconnect)
    window.addEventListener('phrase-gym:receipts', receipts)
    document.addEventListener('visibilitychange', reconnect)
  })
  onUnmounted(() => {
    window.removeEventListener('online', reconnect)
    window.removeEventListener('phrase-gym:receipts', receipts)
    document.removeEventListener('visibilitychange', reconnect)
  })
  return takeTransfers
}
