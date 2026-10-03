<script lang="ts">
  import { Modal } from "$lib/components/ui";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { type WizardView, wizardStep } from "$lib/utils/profileWizard";
  import ImportWizard from "./ImportWizard.svelte";

  const isOpen = $derived(modalStore.current === "profileImport");

  // From the store, so Back (clear) and a failed apply land on the right step.
  const live = $derived.by((): WizardView => {
    const { isApplying, applyResult, currentProfile: profile, validation } = profileStore;
    return { step: wizardStep(isApplying, applyResult, profile, validation), profile, validation, applyResult };
  });

  // Taken by close(): Done and Cancel clear the store, and the exit animation must keep the last step.
  let closingView = $state.raw<WizardView | null>(null);
  const view = $derived(isOpen || !closingView ? live : closingView);

  function close() {
    closingView = live;
    profileStore.clear();
    modalStore.close();
  }
</script>

<!-- The wizard mounts per open, so its choices start fresh. -->
<Modal
  open={isOpen}
  onclose={close}
  size="lg"
  closeOnEscape={view.step !== "applying"}
  closeOnBackdrop={view.step !== "applying"}
>
  <ImportWizard {view} onclose={close} />
</Modal>
