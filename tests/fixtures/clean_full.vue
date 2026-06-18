<template>
  <div>
    <p>Hello World</p>
    <button @click="count++">+</button>
    <span>{{ count }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
</script>
