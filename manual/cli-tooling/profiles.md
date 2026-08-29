---
description: A non technical discussion on advanced identity managed concepts.
---

# Profiles

In reVault a profile is essentially a named master key used to secure your lockboxes.&#x20;

When you initialise a vault an 'default' profile is created.

If you requirements are simple then the default profile is all you need.&#x20;

So why might you want to create additional profiles?

Separate different parts of you life:

You have work and private lockboxes on your PC.

If you uses separate profiles for work and personal lock boxes then if you change jobs you can just delete the personal profile from the vault and no one will have access to any of your personal lock boxes even if you leave some of them on the PC.

If you are a consultant then you might also create separate profiles for each of the organisations you work for. &#x20;

Having separate profiles requires an extra level of care when creating lock boxes, you need to use the --for switch to ensure that you have used the correct profile on a lockbox.

```bash
lbx mygames.lbox create --for personal 
```

### Create a profile

When you first initialise a vault an default profile will have been automatically created.

You can create a new profile called 'personal' by running:

```bash
lbx vault profile personal create
```

### Managing Profiles

You can see the existing profile by running:

```bash
lbx vault profile list
> name      email
  default   your@email
```

You can rename your profile via:

```bash
lbx vault profile default rename work
```

You can update the email associated with a profile by:

```
lbx vault profile work --email my@email
```

## Backup a profile

A profile contains the keys (public and private) used to access your lockboxes. If you loose your profile then you loose access to those lockboxes created for that profile.

The easiest way to backup your profile is to backup your [vault](../)  but you can also directly backup your profile.

```
lbx vault profile [name] backup 
```

DANGER: the profile backup file contains your profile keys - the backup is NOT encrypted.

Anyone with access to your profile backup can open any of your lockboxes created for this profile.

Store you profile securely or better use the vault backup option instead.

### Restore a profile

To restore a profile to your existing vault run:

```
 lbx vault profile [name] restore <backup file>
```

If a profile already exists with the same name then you will need to pass the --overwrite switch:

DANGER - DANGER: overwriting an existing profile destroys its keys removing your access to any lockboxes created using that profile:

```
lbx vault profile [name] restore --overwrite <backup file>
```

