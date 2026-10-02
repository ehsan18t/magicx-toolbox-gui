import { confirmStore } from "$lib/stores/confirm.svelte";
import { tweakOps } from "$lib/stores/tweakOps.svelte";
import { keepCurrentState, restoreTweak, restoreTweaks } from "$lib/stores/tweaksActions.svelte";
import type { TweakDefinition, TweakWithStatus } from "$lib/types";
import { restoreMessage } from "$lib/utils/tweakPresentation";

/** One Restore step, after a confirmation when `ask` is set. */
export async function restoreWithConfirm(def: TweakDefinition, ask: boolean): Promise<void> {
  const ok =
    !ask ||
    (await confirmStore.ask({
      title: `Restore ${def.name}?`,
      message: `This ${def.riskLevel}-risk tweak steps back to the state saved before its last change.`,
      confirmText: "Restore",
      variant: "warning",
    }));
  // A second restore while one is in flight would walk back to the next-older snapshot.
  if (!ok || tweakOps.isRunning(def.id)) return;
  await restoreTweak(def.id, { showToast: true, subject: def.name });
}

export async function restoreAllWithConfirm(title: string, tweaks: TweakWithStatus[]): Promise<void> {
  const ok = await confirmStore.ask({
    title,
    message: restoreMessage(tweaks.length),
    confirmText: "Restore",
    variant: "danger",
  });
  if (ok) await restoreTweaks(tweaks.map((t) => t.definition.id));
}

/** Resolves to whether the snapshot was released; false when declined or refused. */
export async function keepWithConfirm(def: TweakDefinition): Promise<boolean> {
  const ok = await confirmStore.ask({
    title: "Keep the current state?",
    message:
      "This accepts the current state as-is and releases the saved snapshot, so the earlier state can no longer be restored for this tweak.",
    confirmText: "Keep current state",
    variant: "danger",
  });
  return ok && (await keepCurrentState(def.id, { showToast: true, subject: def.name }));
}
