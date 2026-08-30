# reVault terminology

This is the vocabulary standard for the manual, READMEs, CLI help, API guides,
and other material written for users. The published
[glossary](../manual/glossary.md) gives readers the corresponding definitions.

Use product terms with the capitalisation shown here. Use lowercase only for a
command, file name, code identifier, or generic concept.

| Preferred term | Avoid | Usage |
| --- | --- | --- |
| reVault | Revault, ReVault | The product and project. |
| Lockbox | archive, lockbox archive | The portable `.lbox` file. “Archive format” remains appropriate when discussing the file format as a category. |
| Vault | local vault | The encrypted store on one device. Use “local Vault” only when location matters. |
| Vault passphrase | Vault password, Vault pass phrase | The secret that opens the Vault. API identifiers may retain `password` where compatibility requires it. |
| Lockbox password | archive password | An optional password that can open one Lockbox. Profile access remains the normal path. |
| Profile | identity, recipient profile | One of the user's named public and private key identities in the Vault. |
| Profile key | contact key, recipient key | A Profile's key material. Use the more precise public key or private key when it matters. |
| Contact | recipient | Another person's public Profile stored in the Vault. “Recipient” is acceptable for the role in a specific sharing operation. |
| credential | password, key | The general category for material that can provide access. Name the specific Profile key, Lockbox password, or Vault passphrase when the distinction matters. |
| platform credential store | platform key store, platform secret store, OS key store, Secret Service session | The operating system facility used by Auto Open. Name Credential Manager, Secret Service, or Keychain only when describing a platform implementation. |
| Auto Open | auto-open | The feature that stores the Vault passphrase in the platform credential store. Use `auto-open` only for the CLI command or configuration key. |
| Session Agent | Lockbox Session Agent, open cache | The optional local process that temporarily caches Lockbox content keys. |
| `AgentSession` | agent session | The language binding API used to connect to and control the Session Agent. |
| open Lockbox, cached Lockbox key | Lockbox session, agent session | Describe the Lockbox state or cached key directly. Use `session` only for the CLI command name. |
| content key | archive key, direct key | The internal symmetric key that encrypts a Lockbox. Discuss it only in API, format, security, or Session Agent material where the detail is useful. |
| Key Sharing Service | key service | The reVault service used to exchange Profile public keys. A key server is one server running that service. |

## Core definitions

### Lockbox

A Lockbox is a portable `.lbox` file. It stores compressed, encrypted, and
signed files, variables, Forms, and access information. It is designed to be
copied, shared, backed up, uploaded, or downloaded without a hosted reVault
service.

### Vault

A Vault is the encrypted store reVault maintains on one device. It contains
Profiles, Contacts, reusable Form definitions, and remembered Lockbox access.
It is not a collection of Lockboxes and should not be synchronised as a live
file between devices.

### Profile and Contact

A Profile is one of the user's named public and private key identities. A
Contact is another person's public Profile. Use Profile when discussing the
owner's identity and Contact when discussing someone whose public key has been
saved for sharing.

### Credentials and passwords

Credential is the umbrella term for material that grants access. Do not switch
between credential, key, password, and passphrase as if they were synonyms:

- a Vault passphrase opens the Vault;
- a Profile private key opens Lockboxes granted access to that Profile;
- a Lockbox password is an optional access method for one Lockbox; and
- a remembered Lockbox password is a credential stored inside the Vault.

Profiles are the normal way to create and open Lockboxes. Describe password
access as an option for a recipient whose Contact details are not available,
or as a fallback when the Vault does not contain a usable credential.

### Platform credential store and Auto Open

The platform credential store is the operating system facility in which Auto
Open can store the Vault passphrase. On Windows it is Credential Manager, on
Linux it is normally Secret Service, and on macOS it is Keychain. A desktop or
D-Bus session may be required to reach the store, but it is not itself a
credential store or a reVault session.

### Session Agent and `AgentSession`

The Session Agent is the optional process for one user that temporarily caches
Lockbox content keys. `AgentSession` is the language binding API that
applications use when Lockbox access needs to be shared across processes or
process restarts. Describe each entry as an open Lockbox or cached Lockbox key,
not as a separate agent session.

These are separate from Auto Open. Asking `AgentSession` to close a Lockbox
clears its cached content key. It does not delete a persistent credential from
the Vault or the Vault passphrase from the platform credential store.

## Technical format terms

Use the following terms in format and API documentation when the detail is
needed:

- **Lockbox ID:** the public random identifier in a Lockbox header, used for
  cache lookup and Vault records.
- **content key:** the random symmetric key that encrypts the private content
  of one Lockbox.
- **access directory:** the Lockbox metadata containing the entries that allow
  authorised Profile keys or Lockbox passwords to unwrap the content key.

Do not introduce these details into introductory material unless they explain
a benefit visible to users or a security decision.

## Review checklist

Before publishing documentation or help text:

- check new wording against the preferred terms table;
- search changed files for the discouraged alternatives;
- keep compatibility names such as `LOCKBOX_VAULT_PASSWORD` and API methods
  containing `password`, but describe their value as the Vault passphrase; and
- use Credential Manager, Secret Service, or Keychain only when the text is
  explaining behaviour specific to that platform.
