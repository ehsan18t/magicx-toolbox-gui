// corpus.json is generated from the compiled catalogue: `cargo test dump_preview_corpus -- --ignored` in src-tauri.
import type { AppView, CategoryMeta, TweakView } from "$lib/types";
import corpus from "./corpus.json";

export type CorpusTweak = Omit<TweakView, "availability" | "supported"> & { surface: { id: string; name: string }[] };
export type CorpusApp = Omit<AppView, "remove_availability" | "install_availability" | "supported">;

const typed = corpus as unknown as { categories: CategoryMeta[]; tweaks: CorpusTweak[]; apps: CorpusApp[] };
export const { categories, tweaks, apps } = typed;
