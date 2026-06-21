<template>
  <div v-html="raw">flagged, no comment</div>
  <!-- vuer-ignore[no-v-html] -->
  <div v-html="trusted">silenced by previous-line comment</div>
</template>

<script setup lang="ts">
const raw = '<b>x</b>'
const trusted = 'safe'

el.innerHTML = raw  // will be flagged (no ignore)
el.innerHTML = trusted  // vuer-ignore[no-inner-html]
</script>
