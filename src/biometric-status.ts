import type { BiometricSigningEntitlements } from './storage';

export function defaultDataProtectionGroupDeclared(
  entitlements: BiometricSigningEntitlements | null | undefined,
): boolean | null {
  if (!entitlements) return null;
  return entitlements.application_identifier || entitlements.keychain_access_groups;
}
