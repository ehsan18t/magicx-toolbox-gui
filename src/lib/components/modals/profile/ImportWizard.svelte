<script lang="ts" module>
  import type { WizardStep, WizardView } from "$lib/utils/profileWizard";

  const STEP_TITLE: Record<WizardStep, string> = {
    select: "Choose a profile file",
    review: "Review and configure",
    applying: "Applying changes…",
    result: "Done",
  };
</script>

<script lang="ts">
  import { PROFILE_EXT } from "$lib/config/app";
  import { bootStore } from "$lib/stores/boot.svelte";
  import { PROFILE_FILE_REJECTED, profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { listenFileDrop } from "$lib/utils/fileDrop";
  import { plural } from "$lib/utils/format";
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import ApplyingStep from "./import/ApplyingStep.svelte";
  import ResultStep from "./import/ResultStep.svelte";
  import ReviewStep from "./import/ReviewStep.svelte";
  import SelectStep from "./import/SelectStep.svelte";
  import WizardHeader from "./WizardHeader.svelte";

  let { view, onclose }: { view: WizardView; onclose: () => void } = $props();

  // A result left by a modal swapped out without close().
  if (!profileStore.currentProfile) profileStore.clear();

  let skipAlreadyApplied = $state(true);
  // Owned here, so the choices survive a failed apply.
  const skipTweakIds = new SvelteSet<string>();
  let isDragOver = $state(false);

  // Drops outside "select" would reset state mid-review or mid-apply.
  onMount(() =>
    listenFileDrop({
      extension: `.${PROFILE_EXT}`,
      onOver: () => (isDragOver = view.step === "select"),
      onLeave: () => (isDragOver = false),
      onDrop: (path) => {
        if (view.step === "select") void profileStore.importProfileFromPath(path);
      },
      onReject: () => {
        if (view.step === "select") toastStore.error(PROFILE_FILE_REJECTED);
      },
    }),
  );

  function toggleSkip(tweakId: string) {
    if (!skipTweakIds.delete(tweakId)) skipTweakIds.add(tweakId);
  }

  function apply() {
    void profileStore.applyProfile({ skipTweakIds: [...skipTweakIds], skipAlreadyApplied });
  }

  async function finish() {
    const result = view.applyResult;
    await bootStore.rescan();
    onclose();
    if (result?.requires_reboot) {
      toastStore.warning("Some changes need a restart to take effect");
    } else {
      toastStore.success(`Applied ${plural(result?.applied_count ?? 0, "tweak")}`);
    }
  }
</script>

<WizardHeader
  title="Import profile"
  icon="mdi:import"
  step={STEP_TITLE[view.step]}
  onclose={view.step !== "applying" ? onclose : undefined}
/>

{#if view.step === "select"}
  <SelectStep
    {isDragOver}
    isImporting={profileStore.isImporting}
    onbrowse={() => void profileStore.importProfile()}
    oncancel={onclose}
  />
{:else if view.step === "review" && view.profile && view.validation}
  <ReviewStep
    profile={view.profile}
    validation={view.validation}
    {skipTweakIds}
    ontoggle={toggleSkip}
    bind:skipAlreadyApplied
    onback={() => profileStore.clear()}
    onapply={apply}
  />
{:else if view.step === "applying"}
  <ApplyingStep progress={profileStore.applyProgress} />
{:else if view.step === "result" && view.applyResult}
  <ResultStep result={view.applyResult} onfinish={finish} />
{/if}
