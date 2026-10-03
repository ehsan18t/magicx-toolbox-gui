<script lang="ts" module>
  type Step = "select" | "review" | "applying" | "result";

  const STEP_TITLE: Record<Step, string> = {
    select: "Choose a profile file",
    review: "Review and configure",
    applying: "Applying changes…",
    result: "Done",
  };
</script>

<script lang="ts">
  import { Modal } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import { bootStore } from "$lib/stores/boot.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { PROFILE_FILE_REJECTED, profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { listenFileDrop } from "$lib/utils/fileDrop";
  import { plural } from "$lib/utils/format";
  import { untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import ApplyingStep from "./import/ApplyingStep.svelte";
  import ResultStep from "./import/ResultStep.svelte";
  import ReviewStep from "./import/ReviewStep.svelte";
  import SelectStep from "./import/SelectStep.svelte";
  import WizardHeader from "./WizardHeader.svelte";

  const isOpen = $derived(modalStore.current === "profileImport");

  // From the store, so Back (clear) and a failed apply land on the right step.
  function readView() {
    const { isApplying, applyResult, currentProfile: profile, validation } = profileStore;
    const step: Step = isApplying ? "applying" : applyResult ? "result" : profile && validation ? "review" : "select";
    return { step, profile, validation, applyResult };
  }

  // Frozen while closed: Done and Cancel clear the store, and the exit animation must keep the last step.
  let lastView = readView();
  const view = $derived.by(() => (isOpen ? (lastView = readView()) : lastView));

  let skipAlreadyApplied = $state(true);
  const skipTweakIds = new SvelteSet<string>();
  let isDragOver = $state(false);

  // Open edge only: tracking store state here would reset options on every import.
  $effect(() => {
    if (!isOpen) return;
    untrack(() => {
      if (!profileStore.currentProfile) profileStore.clear();
      skipAlreadyApplied = true;
      skipTweakIds.clear();
    });
  });

  // Drops outside "select" would reset state mid-review or mid-apply.
  $effect(() => {
    if (!isOpen) return;
    return listenFileDrop({
      extension: `.${PROFILE_EXT}`,
      onOver: () => (isDragOver = view.step === "select"),
      onLeave: () => (isDragOver = false),
      onDrop: (path) => {
        if (view.step === "select") void reportImport(profileStore.importProfileFromPath(path));
      },
      onReject: () => {
        if (view.step === "select") toastStore.error(PROFILE_FILE_REJECTED);
      },
    });
  });

  function close() {
    profileStore.clear();
    modalStore.close();
  }

  async function reportImport(importing: Promise<boolean>) {
    if (!(await importing) && profileStore.importError) toastStore.error(profileStore.importError);
  }

  async function apply() {
    const success = await profileStore.applyProfile({ skipTweakIds: [...skipTweakIds], skipAlreadyApplied });
    if (!success && profileStore.applyError) toastStore.error(profileStore.applyError);
  }

  async function finish() {
    const result = view.applyResult;
    await bootStore.rescan();
    close();
    if (result?.requires_reboot) {
      toastStore.warning("Some changes need a restart to take effect");
    } else {
      toastStore.success(`Applied ${plural(result?.applied_count ?? 0, "tweak")}`);
    }
  }
</script>

<Modal
  open={isOpen}
  onclose={close}
  size="lg"
  closeOnEscape={view.step !== "applying"}
  closeOnBackdrop={view.step !== "applying"}
>
  <WizardHeader
    title="Import profile"
    icon="mdi:import"
    step={STEP_TITLE[view.step]}
    onclose={view.step !== "applying" ? close : undefined}
  />

  {#if view.step === "select"}
    <SelectStep
      {isDragOver}
      isImporting={profileStore.isImporting}
      onbrowse={() => reportImport(profileStore.importProfile())}
      oncancel={close}
    />
  {:else if view.step === "review" && view.profile && view.validation}
    <ReviewStep
      profile={view.profile}
      validation={view.validation}
      {skipTweakIds}
      bind:skipAlreadyApplied
      onback={() => profileStore.clear()}
      onapply={apply}
    />
  {:else if view.step === "applying"}
    <ApplyingStep progress={profileStore.applyProgress} />
  {:else if view.step === "result" && view.applyResult}
    <ResultStep result={view.applyResult} onfinish={finish} />
  {/if}
</Modal>
