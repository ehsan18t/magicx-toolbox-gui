<script lang="ts">
  import ExternalLink from "$lib/components/ExternalLink.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<div
  class="flex h-full w-full items-center justify-center bg-background text-foreground transition-colors duration-300"
>
  <div>
    <h1 class="mb-8 text-4xl font-bold text-foreground select-text">Welcome to Tauri + Svelte</h1>

    <div class="flex justify-center gap-4">
      <ExternalLink href="https://vitejs.dev" class="logo-link">
        <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
      </ExternalLink>
      <ExternalLink href="https://tauri.app" class="logo-link">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
      </ExternalLink>
      <ExternalLink href="https://kit.svelte.dev" class="logo-link">
        <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
      </ExternalLink>
    </div>

    <p class="mb-8 text-foreground-muted">
      Click on the Tauri, Vite, and SvelteKit logos to learn more.
    </p>

    <form class="row mb-8" onsubmit={greet}>
      <input
        id="greet-input"
        placeholder="Enter a name..."
        bind:value={name}
        class="greet-input mr-2 rounded-lg border border-border bg-card px-4 py-2 text-foreground transition-all duration-200 outline-none hover:border-border-hover focus:border-primary"
      />
      <button
        type="submit"
        class="greet-btn cursor-pointer rounded-lg border border-transparent bg-primary px-6 py-2 font-medium text-primary-foreground transition-all duration-200 hover:bg-primary-hover"
      >
        Greet
      </button>
    </form>

    {#if greetMsg}
      <p
        class="result-msg mx-auto max-w-md rounded-lg border border-border bg-card p-4 text-foreground"
      >
        {greetMsg}
      </p>
    {/if}
  </div>
</div>

<style>
  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 0.3s ease;
  }

  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  .logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  :global(.logo-link) {
    display: inline-block;
    transition: transform 0.2s ease;
  }

  :global(.logo-link:hover) {
    transform: translateY(-2px);
  }

  .row {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .greet-input:focus {
    box-shadow: 0 0 0 3px hsl(var(--primary) / 0.1);
  }

  .greet-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px hsl(var(--primary) / 0.3);
  }

  .greet-btn:active {
    transform: translateY(0);
    box-shadow: 0 2px 4px hsl(var(--primary) / 0.2);
  }

  .result-msg {
    animation: fadeInUp 0.3s ease-out;
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Responsive design */
  @media (max-width: 640px) {
    .row {
      flex-direction: column;
    }

    .logo {
      height: 4em;
      padding: 1em;
    }
  }
</style>
