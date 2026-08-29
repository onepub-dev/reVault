---
description: All of the ways you can share a lockbox.
---

# Sharing

Lockboxes are great at keeping your critical data secret but they are also designed to be shared.

You can share a lockbox with an associate by adding their key (public key) to a lockbox.

Public keys ( the public part of a master key) are designed to be shared and it is completely safe to publish them somewhere publicly visible.  (NEVER share a PRIVATE KEY!).

### Trust

The problem with public keys is establishing the owner of a public key, we refer to this a establishing trust - trust that the public key is owned by who you think it is.&#x20;

Establishing trust in a public key is both CRITICAL and difficult. reVault aims to make this process easier but you still need to take a number of actions to ensure that you can trust a public key.&#x20;

Historically, if Alice sent you a public key and said "this is my key", you could often rely on:

* Email address it came from
* Social media accounts
* Websites
* Human-written communications

AI makes impersonation dramatically easier because:

* AI is great at writing convincing Emails (and keeps getting better).
* Voice calls can be cloned.
* Video calls can be deepfaked.
* Websites can be copied.
* Social media accounts can be automated.

So the question is how do I establish trust in a key.

If you don't establish trust in the key then you may be giving your data away to unscrupulous individuals or organisations!

#### The recommend standard: multiple independent trust channels

The way we attempt to establish trust is to use more than a single communications channel.  To achieve this we use two communications channels and two seperate items.

The items are:

* the public key we wish to receive
* a fingerprint of the public key

A fingerprint for a public key is very much like a human fingerprint in that it is unique to the public key. The fingerprint is a large number which is generated via some clever maths that we don't need to worry about.  What the fingerprint allows us to do is check that it belongs to the public key.

Now that we have two separate items we can communication each in turn via different communications channels.



### The gold standard

If security and trust is critical then you should consider exchanging the key and fingerprint IN PERSON as whilst two separate channels being compromised is difficult - it isn't impossible.

### Exchanging Keys

Now we understand the importance of trust we can look at some of the practical methods for sharing keys and their fingerprint.

### Sharing a Key

reVault operates a key server to help solve the trust issue.&#x20;

The reVault server provides one channel of a two channel trust model for the public keys it shares.

NOTE: you should never trust the reVault server on its own - it could be hacked at some point or secretly taken over by a government authority (yes this is a real concern).&#x20;

The reVault Key Server requires a publisher to verify their email address when publishing a key.&#x20;

* When the user issues a publish command they pass in their email address
* the server sends an email to the user containing a validation link.&#x20;
* Clicking the link sends a verification token back to the server causing it to mark the published key as verified (but not trusted).

```
lbx vault publish [identity] 
> Your public key has been published with the following fingerprint:
XX XX XX XX XX XX
Do not send the fingerprint, ask the user to call you to obtain it.
```

The identity must have an email address associated with it. If not the publish command will fail asking the user to update the identity with an email address.

The publish command prints the email address the key was associated with and informs the user to check their inbox.

NOTE: the reVault server is able to establish that the publisher had access to the email inbox at the time the key was published. It cannot:

* legal identity
* employment
* authority
* long term key ownership

### Receiving a Key

The user runs the receive command to accept a published key:

```
lbx vault receive [contact name]
Public Key has been received. You need to verify the Public Key by asking the publisher
for its fingerprint.
You MUST be the ONE TO INITIATE the communication. If the PUBLISHER SENDS you the fingerprint 
DO NOT ACCEPT IT - it may be a phising attack.
Contact the pubisher over a communications channel that you ALREADY trust - not one just supplied.

fingerprint:
```

The public key will be added to the list of contacts in your vault.

Now that you have entered the fingerprint you now have a trusted key - at least as much as you trust the communication channels you used.

If uncertain - go back to the [gold standard](sharing.md#the-gold-standard).

### Sharing

Now that you have a trusted public key you can use it to share a lockbox with the owner.

To create a lockbox for a contact:

```bash
lbx create --for <contact name> shared.lbox
```

To give access to an existing lockbox:

```
lbx access add shared.lbox <contact name>
```

You can add access for any number of contacts to a single lockbox.

You can now safely email your lockbox to your associate(s) in the knowledge that only they can open the lockbox.&#x20;



## Alternate Key Exchange methods

If you don't trust the reVault Key Exchange server you can have your associate export their public key and send it to you.

Export your public key:

```bash
lockbox vault identity export default ./default.pub
> fingerprint: xx xx xx xx...
```

They can now send you their public key via email or similar and then you can call them, using a number you already trust, to obtain the fingerprint.

You then import the public key via:

```
lbx vault contact import alice ./default.pub
> Enter the fingerprint: 
```



