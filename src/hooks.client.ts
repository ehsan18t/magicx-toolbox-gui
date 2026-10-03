// Dev-only browser preview with fixture IPC: `?preview` (admin) or `?preview&user`. DEV-gated so builds drop it.
export async function init(): Promise<void> {
  const params = new URLSearchParams(location.search);
  if (import.meta.env.DEV && params.has("preview")) {
    const { installPreview } = await import("$lib/preview");
    installPreview(!params.has("user"));
  }
}
