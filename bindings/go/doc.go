// Package revault encrypts files, variables, and typed form records in portable
// lockboxes and manages cryptographic keys and local vault metadata.
//
// Create a Vault to reach package services, then create or open a Lockbox.
// Handles with a Close method own native resources and should be closed
// promptly. Secret variables and form fields use callback-scoped accessors so
// callers can avoid retaining plaintext.
//
// Installation, the security model, and complete examples are in the
// repository README: https://github.com/onepub-dev/reVault#readme.
package revault
