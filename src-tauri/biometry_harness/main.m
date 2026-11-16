#import <CommonCrypto/CommonDigest.h>
#import <Security/Security.h>
#import <stdio.h>
#import <stdlib.h>
#import <string.h>

int geld_keychain_entitlement_status(void);
int geld_biometry_available(void);
int geld_store_biometric_key(const uint8_t *bytes, int count);
int geld_load_biometric_key(uint8_t *output, int capacity, int *length);
int geld_delete_biometric_key(void);

static void print_digest(const uint8_t *key, size_t length) {
    uint8_t digest[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256(key, (CC_LONG)length, digest);
    printf("Synthetic key SHA-256: ");
    for (size_t i = 0; i < sizeof(digest); i++) printf("%02x", digest[i]);
    printf("\n");
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s status|store|load|remove\n", argv[0]);
        return 2;
    }
    if (strcmp(argv[1], "status") == 0) {
        printf("Signed application identifier: %s\n",
               geld_keychain_entitlement_status() == 1 ? "present" :
               geld_keychain_entitlement_status() == 0 ? "absent" : "unknown");
        printf("Biometric hardware policy: %s\n", geld_biometry_available() ? "available" : "unavailable");
        puts("No Keychain operation performed.");
        return 0;
    }
    if (strcmp(argv[1], "store") == 0) {
        uint8_t key[32];
        arc4random_buf(key, sizeof(key));
        puts("This will store a random synthetic 32-byte key under this harness's unique service/account.");
        printf("Type STORE to continue: "); fflush(stdout);
        char response[16] = {0};
        if (fgets(response, sizeof(response), stdin) == NULL || strcmp(response, "STORE\n") != 0) {
            memset_s(key, sizeof(key), 0, sizeof(key));
            puts("Cancelled; no Keychain operation performed.");
            return 1;
        }
        int status = geld_store_biometric_key(key, (int)sizeof(key));
        if (status == errSecSuccess) print_digest(key, sizeof(key));
        memset_s(key, sizeof(key), 0, sizeof(key));
        printf("SecItemAdd status: %d\n", status);
        return status == errSecSuccess ? 0 : 1;
    }
    if (strcmp(argv[1], "load") == 0) {
        uint8_t key[32] = {0};
        int length = 0;
        int status = geld_load_biometric_key(key, (int)sizeof(key), &length);
        if (status == errSecSuccess) print_digest(key, (size_t)length);
        memset_s(key, sizeof(key), 0, sizeof(key));
        printf("SecItemCopyMatching status: %d\n", status);
        return status == errSecSuccess ? 0 : 1;
    }
    if (strcmp(argv[1], "remove") == 0) {
        puts("This deletes only this harness's unique service/account.");
        printf("Type REMOVE to continue: "); fflush(stdout);
        char response[16] = {0};
        if (fgets(response, sizeof(response), stdin) == NULL || strcmp(response, "REMOVE\n") != 0) {
            puts("Cancelled; no Keychain operation performed.");
            return 1;
        }
        int status = geld_delete_biometric_key();
        printf("SecItemDelete status: %d\n", status);
        return status == errSecSuccess ? 0 : 1;
    }
    fprintf(stderr, "Unknown command: %s\n", argv[1]);
    return 2;
}
