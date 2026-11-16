#import <Foundation/Foundation.h>
#import <LocalAuthentication/LocalAuthentication.h>
#import <Security/Security.h>
#import <Security/SecTask.h>

#ifndef GELD_KEYCHAIN_SERVICE
#define GELD_KEYCHAIN_SERVICE "app.geldapp.desktop"
#endif
#ifndef GELD_KEYCHAIN_ACCOUNT
#define GELD_KEYCHAIN_ACCOUNT "vault-key-v1"
#endif
#ifndef GELD_KEYCHAIN_REASON
#define GELD_KEYCHAIN_REASON "Unlock your GeldApp vault"
#endif

static NSString *const GeldService = @GELD_KEYCHAIN_SERVICE;
static NSString *const GeldAccount = @GELD_KEYCHAIN_ACCOUNT;

static int entitlement_present(SecTaskRef task, CFStringRef name, CFTypeID expected_type) {
    CFErrorRef error = NULL;
    CFTypeRef value = SecTaskCopyValueForEntitlement(task, name, &error);
    if (value == NULL) {
        if (error != NULL) {
            CFRelease(error);
            return -1;
        }
        return 0;
    }
    if (error != NULL) CFRelease(error);

    int present = 0;
    if (CFGetTypeID(value) == expected_type) {
        if (expected_type == CFStringGetTypeID()) {
            present = CFStringGetLength((CFStringRef)value) > 0;
        } else if (expected_type == CFArrayGetTypeID()) {
            CFArrayRef values = (CFArrayRef)value;
            present = 1;
            for (CFIndex index = 0; index < CFArrayGetCount(values); index++) {
                CFTypeRef member = CFArrayGetValueAtIndex(values, index);
                if (CFGetTypeID(member) != CFStringGetTypeID() ||
                    CFStringGetLength((CFStringRef)member) == 0) {
                    present = -1;
                    break;
                }
            }
            if (CFArrayGetCount(values) == 0) present = 0;
        } else {
            present = -1;
        }
    } else {
        present = -1;
    }
    CFRelease(value);
    return present;
}

// Inspect effective signing metadata only; never query Data Protection
// Keychain or invoke LocalAuthentication here. Bit 0 is app identifier,
// bit 1 keychain-access-groups, bit 2 application-groups; -1 means unknown.
int geld_keychain_signing_entitlements(void) {
    @autoreleasepool {
        SecTaskRef task = SecTaskCreateFromSelf(kCFAllocatorDefault);
        if (task == NULL) return -1;
        int app_id = entitlement_present(
            task, CFSTR("com.apple.application-identifier"), CFStringGetTypeID()
        );
        int keychain_groups = entitlement_present(
            task, CFSTR("keychain-access-groups"), CFArrayGetTypeID()
        );
        int app_groups = entitlement_present(
            task, CFSTR("com.apple.security.application-groups"), CFArrayGetTypeID()
        );
        CFRelease(task);
        if (app_id < 0 || keychain_groups < 0 || app_groups < 0) return -1;
        return (app_id ? 1 : 0) | (keychain_groups ? 2 : 0) | (app_groups ? 4 : 0);
    }
}

int geld_biometry_available(void) {
    @autoreleasepool {
        LAContext *context = [[LAContext alloc] init];
        NSError *error = nil;
        return [context canEvaluatePolicy:LAPolicyDeviceOwnerAuthenticationWithBiometrics error:&error] ? 1 : 0;
    }
}

int geld_store_biometric_key(const uint8_t *bytes, int count) {
    @autoreleasepool {
        if (bytes == NULL || count != 32) return errSecParam;
        CFErrorRef accessError = NULL;
        SecAccessControlRef access = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            kSecAccessControlBiometryCurrentSet,
            &accessError
        );
        if (access == NULL) {
            if (accessError != NULL) CFRelease(accessError);
            return errSecAuthFailed;
        }
        NSMutableData *value = [NSMutableData dataWithBytes:bytes length:(NSUInteger)count];
        NSDictionary *item = @{
            (__bridge id)kSecClass: (__bridge id)kSecClassGenericPassword,
            (__bridge id)kSecAttrService: GeldService,
            (__bridge id)kSecAttrAccount: GeldAccount,
            (__bridge id)kSecUseDataProtectionKeychain: @YES,
            (__bridge id)kSecAttrSynchronizable: @NO,
            (__bridge id)kSecAttrAccessControl: (__bridge id)access,
            (__bridge id)kSecValueData: value,
        };
        OSStatus status = SecItemAdd((__bridge CFDictionaryRef)item, NULL);
        volatile uint8_t *wipe = (volatile uint8_t *)value.mutableBytes;
        for (NSUInteger index = 0; index < value.length; index++) wipe[index] = 0;
        CFRelease(access);
        return (int)status;
    }
}

int geld_load_biometric_key(uint8_t *output, int capacity, int *length) {
    @autoreleasepool {
        if (output == NULL || length == NULL || capacity < 32) return errSecParam;
        LAContext *context = [[LAContext alloc] init];
        context.localizedReason = @GELD_KEYCHAIN_REASON;
        context.touchIDAuthenticationAllowableReuseDuration = 0;
        NSError *availabilityError = nil;
        if (![context canEvaluatePolicy:LAPolicyDeviceOwnerAuthenticationWithBiometrics error:&availabilityError]) {
            return errSecAuthFailed;
        }
        NSDictionary *query = @{
            (__bridge id)kSecClass: (__bridge id)kSecClassGenericPassword,
            (__bridge id)kSecAttrService: GeldService,
            (__bridge id)kSecAttrAccount: GeldAccount,
            (__bridge id)kSecUseDataProtectionKeychain: @YES,
            (__bridge id)kSecReturnData: @YES,
            (__bridge id)kSecMatchLimit: (__bridge id)kSecMatchLimitOne,
            (__bridge id)kSecUseAuthenticationContext: context,
        };
        CFTypeRef result = NULL;
        OSStatus status = SecItemCopyMatching((__bridge CFDictionaryRef)query, &result);
        if (status != errSecSuccess) return (int)status;
        NSMutableData *data = [NSMutableData dataWithData:CFBridgingRelease(result)];
        if (![data isKindOfClass:[NSData class]] || data.length != 32) return errSecDecode;
        [data getBytes:output length:data.length];
        *length = (int)data.length;
        volatile uint8_t *wipe = (volatile uint8_t *)data.mutableBytes;
        for (NSUInteger index = 0; index < data.length; index++) wipe[index] = 0;
        return errSecSuccess;
    }
}

int geld_delete_biometric_key(void) {
    @autoreleasepool {
        NSDictionary *query = @{
            (__bridge id)kSecClass: (__bridge id)kSecClassGenericPassword,
            (__bridge id)kSecAttrService: GeldService,
            (__bridge id)kSecAttrAccount: GeldAccount,
            (__bridge id)kSecUseDataProtectionKeychain: @YES,
            (__bridge id)kSecMatchLimit: (__bridge id)kSecMatchLimitOne,
        };
        OSStatus status = SecItemDelete((__bridge CFDictionaryRef)query);
        return status == errSecItemNotFound ? errSecSuccess : (int)status;
    }
}
