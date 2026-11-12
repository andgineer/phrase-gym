<script setup lang="ts">
import { computed } from 'vue'
import { emphasisSegments } from '../emphasis'

const props = defineProps<{ text: string }>()
const segments = computed(() => emphasisSegments(props.text))
</script>

<template>
  <template v-for="(segment, index) in segments" :key="index"
    ><strong v-if="segment.bold">{{ segment.text }}</strong
    ><template v-else>{{ segment.text }}</template></template
  >
</template>
