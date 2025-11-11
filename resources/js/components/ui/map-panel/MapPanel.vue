<script setup lang="ts">

import { Card } from '@/components/ui/card';
</script>

<template>
  <Card class="h-full overflow-y-auto bg-neutral-50 pb-0 dark:bg-background">
      <slot />
  </Card>
</template>

<style scoped>

</style>
