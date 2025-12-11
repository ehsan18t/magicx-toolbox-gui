<script lang="ts">
  import "@/app.css";
  import DebugPanel from "@/lib/components/DebugPanel.svelte";
  import TitleBar from "@/lib/components/TitleBar.svelte";
  import { colorSchemeStore } from "@/lib/stores/colorScheme";
  import { themeStore } from "@/lib/stores/theme";
  import { onMount } from "svelte";

  const { children } = $props();
  onMount(() => {
    themeStore.init();
    colorSchemeStore.init();
  });
</script>

<TitleBar />
<!-- TitleBar height=h-10 == 2.5rem -->
<main class="h-[calc(100dvh-2.5rem)] w-full overflow-auto">
  {@render children()}
</main>
<DebugPanel />
