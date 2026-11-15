#import <Foundation/Foundation.h>
#import <LocalAuthentication/LocalAuthentication.h>
#import <Security/Security.h>

static NSString *const GeldService = @"app.geldapp.desktop";
static NSString *const GeldAccount = @"vault-key-v1";

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
        context.localizedReason = @"Unlock your GeldApp vault";
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
