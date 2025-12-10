<script lang="ts">
  import { pendingRebootCount, pendingRebootStore, pendingRebootTweaks } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";

  let showDetails = $state(false);
</script>

{#if $pendingRebootCount > 0}
  <div class="reboot-banner">
    <div class="banner-content">
      <div class="banner-icon">
        <Icon icon="mdi:restart-alert" width="24" />
      </div>
      <div class="banner-text">
        <span class="banner-title">Restart Required</span>
        <span class="banner-subtitle">
          {$pendingRebootCount} tweak{$pendingRebootCount === 1 ? "" : "s"} need a system restart to take
          effect
        </span>
      </div>
    </div>

    <div class="banner-actions">
      <button class="details-btn" onclick={() => (showDetails = !showDetails)}>
        <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
        {showDetails ? "Hide" : "Details"}
      </button>
      <button
        class="dismiss-btn"
        onclick={() => pendingRebootStore.clear()}
        title="Dismiss (changes still apply after restart)"
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    </div>
  </div>

  {#if showDetails}
    <div class="pending-list">
      <ul>
        {#each $pendingRebootTweaks as tweak (tweak.definition.id)}
          <li>
            <Icon icon="mdi:restart" width="14" />
            {tweak.definition.name}
          </li>
        {/each}
      </ul>
    </div>
  {/if}
{/if}

<style>
  .reboot-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: linear-gradient(135deg, hsl(24 94% 50% / 0.15), hsl(45 93% 47% / 0.1));
    border: 1px solid hsl(24 94% 50% / 0.3);
    border-radius: 8px;
    margin-bottom: 16px;
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .banner-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: hsl(24 94% 50% / 0.2);
    border-radius: 8px;
    color: hsl(24 94% 50%);
  }

  .banner-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .banner-title {
    font-size: 14px;
    font-weight: 600;
    color: hsl(var(--foreground));
  }

  .banner-subtitle {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }

  .banner-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .details-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    background: hsl(var(--muted));
    border: none;
    border-radius: 6px;
    color: hsl(var(--foreground));
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .details-btn:hover {
    background: hsl(var(--muted) / 0.8);
  }

  .dismiss-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .dismiss-btn:hover {
    background: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  .pending-list {
    background: hsl(var(--muted) / 0.3);
    border: 1px solid hsl(var(--border));
    border-radius: 6px;
    padding: 12px 16px;
    margin-bottom: 16px;
    margin-top: -8px;
  }

  .pending-list ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .pending-list li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: hsl(var(--foreground) / 0.8);
  }

  .pending-list li :global(svg) {
    color: hsl(24 94% 50%);
  }
</style>
