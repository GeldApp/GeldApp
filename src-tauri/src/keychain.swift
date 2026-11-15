import Foundation
import LocalAuthentication
import Security

private let service = "app.geldapp.desktop"
private let account = "vault-key-v1"

@_cdecl("geld_biometry_available")
public func geldBiometryAvailable() -> Int32 {
    let context = LAContext()
    var error: NSError?
    return context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) ? 1 : 0
}

@_cdecl("geld_store_biometric_key")
public func geldStoreBiometricKey(_ bytes: UnsafePointer<UInt8>?, _ count: Int32) -> Int32 {
    guard let bytes, count == 32 else { return errSecParam }
    var accessError: Unmanaged<CFError>?
    guard let access = SecAccessControlCreateWithFlags(
        kCFAllocatorDefault,
        kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        .biometryCurrentSet,
        &accessError
    ) else { return errSecAuthFailed }

    let value = Data(bytes: bytes, count: Int(count))
    let item: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrService: service,
        kSecAttrAccount: account,
        kSecUseDataProtectionKeychain: true,
        kSecAttrSynchronizable: false,
        kSecAttrAccessControl: access,
        kSecValueData: value,
    ]
    // Never update an existing item in place: that could preserve a weaker ACL.
    // A duplicate must fail closed instead of reusing a credential with unknown policy.
    return SecItemAdd(item as CFDictionary, nil)
}

@_cdecl("geld_load_biometric_key")
public func geldLoadBiometricKey(_ output: UnsafeMutablePointer<UInt8>?, _ capacity: Int32, _ length: UnsafeMutablePointer<Int32>?) -> Int32 {
    guard let output, let length, capacity >= 32 else { return errSecParam }
    let context = LAContext()
    context.localizedReason = "Unlock your GeldApp vault"
    context.touchIDAuthenticationAllowableReuseDuration = 0
    var availabilityError: NSError?
    guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &availabilityError) else {
        return errSecAuthFailed
    }
    let query: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrService: service,
        kSecAttrAccount: account,
        kSecUseDataProtectionKeychain: true,
        kSecReturnData: true,
        kSecMatchLimit: kSecMatchLimitOne,
        kSecUseAuthenticationContext: context,
    ]
    var result: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &result)
    guard status == errSecSuccess else { return status }
    guard let data = result as? Data, data.count == 32 else { return errSecDecode }
    data.copyBytes(to: output, count: data.count)
    length.pointee = Int32(data.count)
    return errSecSuccess
}

@_cdecl("geld_delete_biometric_key")
public func geldDeleteBiometricKey() -> Int32 {
    let query: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrService: service,
        kSecAttrAccount: account,
        kSecUseDataProtectionKeychain: true,
    ]
    let status = SecItemDelete(query as CFDictionary)
    return status == errSecItemNotFound ? errSecSuccess : status
}
