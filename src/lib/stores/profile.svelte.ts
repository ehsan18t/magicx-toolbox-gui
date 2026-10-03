import { appDataPath, joinPath, pickFile, pickFolder, pickSavePath } from "$lib/api/platform";
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
import { toastStore } from "./toast.svelte";

const PROFILE_FILTERS = [{ name: "MagicX Profile", extensions: [PROFILE_EXT] }];
const DEFAULT_PROFILE_SUBDIR = "profiles";

/** The toast for a dropped file that is not a profile. */
export const PROFILE_FILE_REJECTED = `Only .${PROFILE_EXT} profile files can be imported.`;

const profileFileName = (name: string) => `${name.replace(/[^a-z0-9\-_]/gi, "")}.${PROFILE_EXT}`;

let isExporting = $state(false);
let isImporting = $state(false);

let savedProfiles = $state.raw<ProfileMetadata[]>([]);
let isLoadingSavedProfiles = $state(false);
let savedProfilesError = $state<string | null>(null);
// A re-entry guard only: nothing renders it.
let isDeleting = false;
// Drops a saved-profiles response superseded by a later directory change.
let loadSeq = 0;

const profileDir = new PersistentStore<string | null>(STORAGE_KEYS.profileDir, null, (stored) =>
  stored === null || (typeof stored === "string" && stored !== "") ? stored : undefined,
);

let isApplying = $state(false);
let applyProgress = $state<{ current: number; total: number } | null>(null);

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
  currentProfile = null;
  validation = null;
  applyResult = null;
  try {
    const filePath = await pick();
    if (!filePath) return false;
    [currentProfile, validation] = await profileApi.importProfile(filePath);
    return true;
  } catch (error) {
    toastStore.failure("Failed to import profile", error);
    return false;
  } finally {
    isImporting = false;
  }
}

function setProfileDir(path: string | null) {
  profileDir.value = path;
  void loadSavedProfiles();
}

export const profileStore = {
  get isExporting() {
    return isExporting;
  },
  get isImporting() {
    return isImporting;
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
  get profileDir() {
    return profileDir.value;
  },
  get isApplying() {
    return isApplying;
  },
  get applyProgress() {
    return applyProgress;
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

  setProfileDir,

  /** Asks for a folder to list profiles from; the chosen path, or null when cancelled or the picker failed. */
  async chooseFolder(): Promise<string | null> {
    try {
      const selected = await pickFolder("Select a profile folder");
      if (!selected) return null;
      setProfileDir(selected);
      return selected;
    } catch (error) {
      toastStore.failure("Failed to open the folder picker", error);
      return null;
    }
  },

  /** Asks where to save, then exports the selected tweaks. A failure is toasted; false also on cancel. */
  async exportProfile(name: string, tweakIds: string[], options?: ExportOptions): Promise<boolean> {
    if (isExporting) return false;
    isExporting = true;
    try {
      const filePath = await pickSavePath(`${name.toLowerCase().replace(/\s+/g, "-")}.${PROFILE_EXT}`, PROFILE_FILTERS);
      if (!filePath) return false;
      await profileApi.exportProfile(filePath, name, tweakIds, options);
      void loadSavedProfiles();
      return true;
    } catch (error) {
      toastStore.failure("Failed to export profile", error);
      return false;
    } finally {
      isExporting = false;
    }
  },

  /** A failure is toasted. */
  async deleteProfile(name: string): Promise<boolean> {
    if (isDeleting) return false;
    isDeleting = true;
    try {
      await profileApi.deleteSavedProfile(name, profileDir.value);
      await loadSavedProfiles();
      return true;
    } catch (error) {
      toastStore.failure("Failed to delete profile", error);
      return false;
    } finally {
      isDeleting = false;
    }
  },

  /** Opens a file dialog, then loads and validates the chosen profile. Each import toasts its failure. */
  importProfile(): Promise<boolean> {
    return importFrom(() => pickFile(PROFILE_FILTERS));
  },

  /** For a dropped file. */
  importProfileFromPath(filePath: string): Promise<boolean> {
    return importFrom(async () => filePath);
  },

  /** A saved profile by name, from the chosen folder or the app data default. */
  importSaved(name: string): Promise<boolean> {
    return importFrom(async () => {
      const dir = profileDir.value ?? (await appDataPath(DEFAULT_PROFILE_SUBDIR));
      return joinPath(dir, profileFileName(name));
    });
  },

  /** A failure is toasted; a partial apply is false with its result kept. */
  async applyProfile(options?: ApplyOptions): Promise<boolean> {
    if (isApplying || !currentProfile) return false;
    isApplying = true;
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
      toastStore.failure("Failed to apply profile", error);
      return false;
    } finally {
      isApplying = false;
    }
  },

  /** Drops results; in-flight flags stay, so the re-entry guards hold. */
  clear() {
    applyProgress = null;
    currentProfile = null;
    validation = null;
    applyResult = null;
  },
};
