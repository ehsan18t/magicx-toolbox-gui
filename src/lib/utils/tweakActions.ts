import { confirm } from "$lib/stores/confirm.svelte";
import { keepCurrentState, loadingStore, revertTweak } from "$lib/stores/tweaks.svelte";
import type { TweakDefinition } from "$lib/types";

/** One Restore step, after a confirmation when `ask` is set. */
export async function restoreTweak(def: TweakDefinition, ask: boolean): Promise<void> {
  const ok =
    !ask ||
    (await confirm({
      title: `Restore ${def.name}?`,
      message: `This ${def.risk_level}-risk tweak steps back to the state saved before its last change.`,
      confirmText: "Restore",
      variant: "warning",
    }));
  // A second restore while one is in flight would walk back to the next-older snapshot.
  if (!ok || loadingStore.isLoading(def.id)) return;
  await revertTweak(def.id, { showToast: true, tweakName: def.name });
}

/** Resolves to whether the snapshot was released; false when declined or refused. */
export async function keepWithConfirm(def: TweakDefinition): Promise<boolean> {
  const ok = await confirm({
    title: "Keep the current state?",
    message:
      "This accepts the current state as-is and releases the saved snapshot, so the earlier state can no longer be restored for this tweak.",
    confirmText: "Keep current state",
    variant: "danger",
  });
  return ok && (await keepCurrentState(def.id, { showToast: true, tweakName: def.name }));
}
