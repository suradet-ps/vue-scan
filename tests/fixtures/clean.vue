<template>
  <div>
    <h1>Hello World</h1>
    <p>{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const message = ref<string>('Hello, Vue!')
</script>
