<script lang="ts">
  import { Button, IconTile, ModalBody, ModalFooter, Spinner } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import ProfileFileNote from "../ProfileFileNote.svelte";

  interface Props {
    isDragOver: boolean;
    isImporting: boolean;
    onbrowse: () => void;
    oncancel: () => void;
  }

  let { isDragOver, isImporting, onbrowse, oncancel }: Props = $props();
</script>

<ModalBody class="animate-fade-in space-y-4">
  <button
    type="button"
    class={[
      "flex w-full cursor-pointer flex-col items-center gap-3 rounded-lg border-2 border-dashed p-10 transition-colors",
      isDragOver ? "border-accent bg-muted" : "border-border hover:border-accent hover:bg-muted",
    ]}
    onclick={onbrowse}
  >
    <IconTile icon="mdi:file-import" size="2xl" shape="circle" tone={isDragOver ? "accent" : "neutral"} />
    <span>
      <span class="block font-medium">{isDragOver ? "Drop the file here" : "Browse for a profile"}</span>
      <span class="mt-1 block text-sm text-foreground-muted">or drop a .{PROFILE_EXT} file here</span>
    </span>
  </button>

  {#if isImporting}
    <Spinner class="flex justify-center py-4 text-sm text-foreground-muted">Loading profile…</Spinner>
  {/if}

  <ProfileFileNote>
    lists tweaks and the options to set. It is checked against this PC's Windows version and this app before anything
    changes.
  </ProfileFileNote>
</ModalBody>

<ModalFooter>
  <Button variant="secondary" onclick={oncancel}>Cancel</Button>
  <Button variant="primary" icon="mdi:folder-open" loading={isImporting} onclick={onbrowse}>Browse files</Button>
</ModalFooter>
