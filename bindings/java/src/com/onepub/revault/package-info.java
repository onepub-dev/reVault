/**
 * Encrypts files, variables, and typed form records in portable reVault
 * lockboxes and manages keys and local vault metadata.
 *
 * <p>{@link com.onepub.revault.Revault} is the main entry point. Owned handles
 * implement {@link java.lang.AutoCloseable}; use try-with-resources to release
 * native state promptly. Secret accessors use callback scope to reduce the
 * lifetime of plaintext in managed memory.
 *
 * <p>See the <a href="https://github.com/onepub-dev/reVault#readme">repository
 * README</a> for installation, security guidance, and complete examples.
 */
package com.onepub.revault;
