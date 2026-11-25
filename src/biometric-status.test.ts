import { describe, expect, it } from 'vitest';
import { defaultDataProtectionGroupDeclared } from './biometric-status';

describe('Data Protection Keychain signing prerequisites', () => {
  it('accepts either signed source of the default access group', () => {
    expect(defaultDataProtectionGroupDeclared({
      application_identifier: true,
      keychain_access_groups: false,
      application_groups: false,
    })).toBe(true);
    expect(defaultDataProtectionGroupDeclared({
      application_identifier: false,
      keychain_access_groups: true,
      application_groups: false,
    })).toBe(true);
  });

  it('does not treat application groups alone as a default keychain group', () => {
    expect(defaultDataProtectionGroupDeclared({
      application_identifier: false,
      keychain_access_groups: false,
      application_groups: true,
    })).toBe(false);
  });

  it('keeps unavailable diagnostics unknown', () => {
    expect(defaultDataProtectionGroupDeclared(null)).toBeNull();
  });
});
