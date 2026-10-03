import * as profileApi from "$lib/api/profile";
import { PROFILE_EXT, STORAGE_KEYS } from "$lib/config/app";
import type {
  ApplyOptions,
  ConfigurationProfile,
  ExportOptions,
  ProfileApplyResult,
  ProfileMetadata,
  ProfileValidation,
} from "$lib/types";
import { errorMessage } from "$lib/utils/error";
import { logError } from "$lib/utils/logger";
import { PersistentStore } from "$lib/utils/persistentStore.svelte";
import { appDataDir, join } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";

const PROFILE_FILTERS = [{ name: "MagicX Profile", extensions: [PROFILE_EXT] }];
const DEFAULT_PROFILE_SUBDIR = "profiles";

const profileFileName = (name: string) => `${name.replace(/[^a-z0-9\-_]/gi, "")}.${PROFILE_EXT}`;

let isExporting = $state(false);
let exportError = $state<string | null>(null);

let isImporting = $state(false);
let importError = $state<string | null>(null);

let savedProfiles = $state.raw<ProfileMetadata[]>([]);
let isLoadingSavedProfiles = $state(false);
let savedProfilesError = $state<string | null>(null);
// A re-entry guard only: nothing renders it.
let isDeleting = false;
let deleteError = $state<string | null>(null);
// Drops a saved-profiles response superseded by a later directory change.
let loadSeq = 0;

const profileDir = new PersistentStore<string | null>(STORAGE_KEYS.profileDir, null);

let isApplying = $state(false);
let applyProgress = $state<{ current: number; total: number } | null>(null);
let applyError = $state<string | null>(null);

let currentProfile = $state<ConfigurationProfile | null>(null);
let validation = $state<ProfileValidation | null>(null);
let applyResult = $state<ProfileApplyResult | null>(null);

async function loadSavedProfiles() {
  const mine = ++loadSeq;
  isLoadingSavedProfiles = true;
  savedProfilesError = null;
  try {
    const list = await profileApi.getSavedProfiles(profileDir.value);
    if (mine === loadSeq) savedProfiles = list;
  } catch (error) {
    if (mine !== loadSeq) return;
    logError("Failed to load saved profiles", error);
    savedProfilesError = errorMessage(error);
  } finally {
    if (mine === loadSeq) isLoadingSavedProfiles = false;
  }
}

/** Loads and validates the profile at the path `pick` resolves to; null is a cancelled dialog. */
async function importFrom(pick: () => Promise<string | null>): Promise<boolean> {
  if (isImporting) return false;
  isImporting = true;
  importError = null;
  currentProfile = null;
  validation = null;
  applyResult = null;
  try {
    const filePath = await pick();
    if (!filePath) return false;
    [currentProfile, validation] = await profileApi.importProfile(filePath);
    return true;
  } catch (error) {
    logError("Failed to import profile", error);
    importError = errorMessage(error);
    return false;
  } finally {
    isImporting = false;
  }
}

export const profileStore = {
  get isExporting() {
    return isExporting;
  },
  get exportError() {
    return exportError;
  },
  get isImporting() {
    return isImporting;
  },
  get importError() {
    return importError;
  },
  get savedProfiles() {
    return savedProfiles;
  },
  get isLoadingSavedProfiles() {
    return isLoadingSavedProfiles;
  },
  get savedProfilesError() {
    return savedProfilesError;
  },
  get deleteError() {
    return deleteError;
  },
  get profileDir() {
    return profileDir.value;
  },
  get isApplying() {
    return isApplying;
  },
  get applyProgress() {
    return applyProgress;
  },
  get applyError() {
    return applyError;
  },
  get currentProfile() {
    return currentProfile;
  },
  get validation() {
    return validation;
  },
  get applyResult() {
    return applyResult;
  },

  setProfileDir(path: string | null) {
    profileDir.value = path;
    void loadSavedProfiles();
  },

  /** Asks where to save, then exports the selected tweaks. */
  async exportProfile(name: string, tweakIds: string[], options?: ExportOptions): Promise<boolean> {
    if (isExporting) return false;
    isExporting = true;
    exportError = null;
    try {
      const filePath = await save({
        defaultPath: `${name.toLowerCase().replace(/\s+/g, "-")}.${PROFILE_EXT}`,
        filters: PROFILE_FILTERS,
      });
      if (!filePath) return false;
      await profileApi.exportProfile(filePath, name, tweakIds, options);
      void loadSavedProfiles();
      return true;
    } catch (error) {
      logError("Failed to export profile", error);
      exportError = errorMessage(error);
      return false;
    } finally {
      isExporting = false;
    }
  },

  async deleteProfile(name: string): Promise<boolean> {
    if (isDeleting) return false;
    isDeleting = true;
    deleteError = null;
    try {
      await profileApi.deleteSavedProfile(name, profileDir.value);
      await loadSavedProfiles();
      return true;
    } catch (error) {
      logError("Failed to delete profile", error);
      deleteError = errorMessage(error);
      return false;
    } finally {
      isDeleting = false;
    }
  },

  /** Opens a file dialog, then loads and validates the chosen profile. */
  importProfile(): Promise<boolean> {
    return importFrom(async () => {
      const filePath = await open({ multiple: false, filters: PROFILE_FILTERS });
      return typeof filePath === "string" ? filePath : null;
    });
  },

  /** For a dropped file. */
  importProfileFromPath(filePath: string): Promise<boolean> {
    return importFrom(async () => filePath);
  },

  /** A saved profile by name, from the chosen folder or the app data default. */
  importSaved(name: string): Promise<boolean> {
    return importFrom(async () => {
      const dir = profileDir.value ?? (await join(await appDataDir(), DEFAULT_PROFILE_SUBDIR));
      return join(dir, profileFileName(name));
    });
  },

  async applyProfile(options?: ApplyOptions): Promise<boolean> {
    if (isApplying || !currentProfile) return false;
    isApplying = true;
    applyError = null;
    applyResult = null;

    const total =
      validation?.preview.filter(
        (p) =>
          p.applicable &&
          !(options?.skipAlreadyApplied && p.already_applied) &&
          !options?.skipTweakIds?.includes(p.tweak_id),
      ).length ?? 0;
    applyProgress = { current: 0, total };

    try {
      const result = await profileApi.applyProfile(currentProfile, options);
      applyResult = result;
      applyProgress = { current: result.applied_count, total };
      return result.success;
    } catch (error) {
      logError("Failed to apply profile", error);
      applyError = errorMessage(error);
      return false;
    } finally {
      isApplying = false;
    }
  },

  /** Drops results and errors; in-flight flags stay, so the re-entry guards hold. */
  clear() {
    exportError = null;
    importError = null;
    applyProgress = null;
    applyError = null;
    currentProfile = null;
    validation = null;
    applyResult = null;
  },
};
