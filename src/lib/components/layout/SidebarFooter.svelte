<script lang="ts">
  import { Dot, IconButton } from "$lib/components/ui";
  import { type IconName, TONE_FILL } from "$lib/design";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { updateStore } from "$lib/stores/update.svelte";

  const items: { label: string; icon: IconName; open: () => void; dot: boolean; active: boolean }[] = $derived([
    {
      label: updateStore.isAvailable ? "Update available" : "Updates",
      icon: "mdi:update",
      open: () => modalStore.open("update"),
      dot: updateStore.isAvailable,
      active: false,
    },
    {
      label: "Settings",
      icon: "mdi:cog-outline",
      open: () => navigationStore.navigateToTab("settings"),
      dot: false,
      active: navigationStore.activeTab === "settings",
    },
    {
      label: "About",
      icon: "mdi:information-outline",
      open: () => modalStore.open("about"),
      dot: false,
      active: false,
    },
  ]);
</script>

<div
  class="flex shrink-0 gap-0.5 border-t border-border px-1.5 py-1.5 {sidebarStore.isOpen
    ? 'flex-row justify-around'
    : 'flex-col'}"
>
  {#each items as item (item.label)}
    <div class="relative shrink-0 {sidebarStore.isOpen ? 'flex-1' : ''}">
      <IconButton
        icon={item.icon}
        tooltip={item.label}
        active={item.active}
        aria-current={item.active ? "page" : undefined}
        class="h-9 w-full {item.active ? 'bg-muted' : ''}"
        onclick={() => {
          sidebarStore.closeOverlay();
          item.open();
        }}
      />
      {#if item.dot}
        <Dot
          fill={TONE_FILL.success}
          class="pointer-events-none absolute top-1.5 right-1/2 translate-x-3 animate-pop-in ring-2 ring-background"
        />
      {/if}
    </div>
  {/each}
</div>
