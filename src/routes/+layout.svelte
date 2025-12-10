<script lang="ts">
  import "@/app.css";
  import TitleBar from "@/lib/components/TitleBar.svelte";
  import { themeStore } from "@/lib/stores/theme";
  import { onMount } from "svelte";

  const { children } = $props();
  onMount(() => {
    themeStore.init();
  });
</script>

<TitleBar />
<!-- TitleBar height=h-10 == 2.5rem -->
<main class="h-[calc(100dvh-2.5rem)] w-full overflow-auto">
  {@render children()}
</main>
