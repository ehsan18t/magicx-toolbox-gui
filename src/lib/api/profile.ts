// The profile backend does not exist yet (docs/spec/profile-v1.md). The UI stays, so the feature reads as
// coming back; every call rejects here, at one choke point, and no invoke() reaches an unregistered command.
import type {
  ProfileMetadata,
  ConfigurationProfile,
  ProfileValidation,
  ProfileApplyResult,
  ExportOptions,
  ApplyOptions,
} from "$lib/types";

const PROFILE_UNAVAILABLE_MESSAGE = "Profiles are temporarily unavailable while the system is rebuilt.";

function profileBackendUnavailable(): Promise<never> {
  return Promise.reject(new Error(PROFILE_UNAVAILABLE_MESSAGE));
}

export async function exportProfile(
  _filePath: string,
  _profileName: string,
  _tweakIds: string[],
  _options?: ExportOptions,
): Promise<void> {
  return profileBackendUnavailable();
}

/** Loads and validates the profile at the path. */
export async function importProfile(_filePath: string): Promise<[ConfigurationProfile, ProfileValidation]> {
  return profileBackendUnavailable();
}

export async function applyProfile(
  _profile: ConfigurationProfile,
  _options?: ApplyOptions,
): Promise<ProfileApplyResult> {
  return profileBackendUnavailable();
}

/** From the app data directory, or `customPath` when given. */
export async function getSavedProfiles(_customPath?: string | null): Promise<ProfileMetadata[]> {
  return profileBackendUnavailable();
}

/** `name` without the extension. */
export async function deleteSavedProfile(_name: string, _customPath?: string | null): Promise<void> {
  return profileBackendUnavailable();
}
