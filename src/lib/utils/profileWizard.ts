import type { ConfigurationProfile, ProfileApplyResult, ProfileValidation } from "$lib/types";

export type WizardStep = "select" | "review" | "applying" | "result";

export interface WizardView {
  step: WizardStep;
  profile: ConfigurationProfile | null;
  validation: ProfileValidation | null;
  applyResult: ProfileApplyResult | null;
}

export const wizardStep = (
  isApplying: boolean,
  applyResult: ProfileApplyResult | null,
  profile: ConfigurationProfile | null,
  validation: ProfileValidation | null,
): WizardStep => (isApplying ? "applying" : applyResult ? "result" : profile && validation ? "review" : "select");
