<template>
  <div>
    <div v-html="userInput">Content</div>
    <img :src="imageUrl">
    <div style="color: red; font-size: 14px;">Styled</div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const userInput = ref('<b>Bold</b>')
const imageUrl = ref('/logo.png')

watch(count, (newVal) => {
  console.log('Count changed:', newVal)
})
</script>
