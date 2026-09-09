---
description: "Rotate a Profile and refresh access without losing older Lockboxes."
---

# Rotate Profile keys

Rotation creates a new active key generation while retaining the old generation needed to open existing Lockboxes.

Back up the Profile and inspect its history first:

```bash
lbx vault profile backup ./production-before-rotation.profile-backup --name production
lbx vault profile history production
lbx vault profile rotate production
lbx vault profile history production
```

Rotation does not rewrite every Lockbox automatically. Preview stale access entries:

```bash
lbx access refresh --all production --dry-run
```

Refresh the known Lockboxes after reviewing the plan:

```bash
lbx access refresh --all production
```

Or update one Lockbox:

```bash
lbx project.lbox access refresh production
```

Publish the new public generation and have Contacts verify the new fingerprint through their established channel. Key continuity helps with planned rotation, but recipients should still investigate an unexpected replacement.

{% hint style="warning" %}
Rotation limits future reliance on an older key; it cannot remove data or key material from copies already held by another person or compromised machine.
{% endhint %}

Retain old generations until every required Lockbox has been refreshed and verified. Removing the only generation capable of opening an older copy can make that copy inaccessible.
