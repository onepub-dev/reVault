# Session Management

On platforms that support reVault Auto Open, your vault password is stored in a platform provided key ring. A key ring is a bit like the little shelf you hang your keys on as you enter the house. We place the reVault password on the shelf (the key ring) and then anyone already in the house can unlock the vault.  In this case 'already in the house' means you have access to your vault after you log in to your desktop session (i.e you enter the house by logging in).

Storing your reVault vault password in a platform key ring allows your reVault vault password to be available whilst you are logged in to your desktop environment.&#x20;

As the vault password is available during your logged in session, you don't need to re-enter your password when doing most lockbox activity.

With Auto Open enabled:

```
lbx add mystuff.lbox readme.md
> You must first open the lockbox.
```

With Auto Open enabled the add just works:

```
lbx add mystuff.lbox readme.md
```

You can see if your platform supports Auto Open by running:

```
lbx doctor
```

### Managing Auto Open

If you work in a particularly sensitive environment then you may want to disable auto open.

You can enable/disable Auto Open via the command:

```
lbx session auto-open off
```

Turning auto open off removes you vault password from the platform key ring.

Auto Open has two levels

* auto open the vault
* auto open lockboxes

You can set the level via:

```
lbx session auto-open vault
lbx session auto-open lockboxes
```

You will need to re-enter your vault password.



The vault password will be re-inserted into the platform key ring the next time you enter your vault password (for any lockbox command).

### Keeping your data secure

When Auto Unlock is enabled all lockboxes on your system are exposed to any user that has access to you logged in session - as such it is CRITICAL that you either log out or lock you desktop session when you walk away from your PC.

### Platform Secure Key Services (SKS)

SKS is reVault umbrella terminology for a number of different technologies provided by different platforms.  Here is the list of specific platform technologies that reVault supports.

|         |                    |
| ------- | ------------------ |
| Windows | Credential Manager |
| LInux   | libsecret          |
| MacOS   | Keychain           |

It is CRITICAL that you backup the pass phrase. If you lose the pass phrase you will lose access to all of you lockboxes.
