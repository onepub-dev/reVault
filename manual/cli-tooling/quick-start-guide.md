# Quick start guide

No cruft just the juicy bits.

This guide focus' on the CLI tooling. You don't need to use the CLI tools as you can perform operations using any of the [language bindings](../apis/revault-api.md). Having said that the CLI tools are a great place to start to familiarize yourself with the core concept or reVault.

Read on if you want to take the CLI out for a drive.

If you don't have cargo (the Rust package manager) follow the cargo install guide:

[https://doc.rust-lang.org/cargo/getting-started/installation.html](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Once cargo is installed

Install reVault:

```
cargo install revault_cli
```

We assume here that the cargo bin directory is on your path.&#x20;

Do the one time initialisation of your vault

```
lbx vault init
```

This will generate a CRITICAL pass phrase - store it someplace VERY safe and do not lose. If you lose it you lose access to all of your lockboxes and we can do nothing to help you!!!!!

Create a lockbox:

```
lbx create mystuff.lbox
```

Set the lockbox as the default (you don't need to do this but it reduces typing).

```
lbx session default mystuff.lbox
```

Open the lockbox:

```
lbx open --duration 1h
```

By default the lockbox will be held open for 15min.

Add a file to a lockbox:

```
lbox add reamde.md
```

Add a directory:

```
lbox add /home/me
```

Add a file, but store it in a directory in the lockbox:

```
lbox add readme.md --to /some/place/in/the/lockbox
```

List the contents of the default lockbox:

```
lbx ls 
```

Find all .md files in the lockbox

```
lbx ls *.md
```

Remove a file from the lockbox:

```
lbx rm readme.md
```

Add a variable:

```
lbox var set DBPORT 80
lbox var set --secret PASSWORD --file /my/db/password.txt
lbox var set name "A developer"
lbox var set /production/DBPORT 80
```

Get a variable&#x20;

```
lbox var get DBPORT
lbox var get --secret PASSWORD
lbox var get /production/DBPORT
```

Delete a variable:

```
lbox var rm DBPORT
```

Export a variable:

```
lbx var export --format posix
DBPORT='80'
lbx var export --format powershell
$env:DBPORT = '80'
```

Create a form definition:

TODO: complete form examples

```
lbx form
```

Share a lockbox:

Have an associate install reVault and then ask the to run:

```
lbx vault publish
```

Ask them for the share code and fingerprint (make certain you initiate the conversation to avoid being being hacked by AI).

On your machine run:

```
lbx vault receive <share-code> <contact name>
```

Enter the fingerprint when prompted.

Now give your associate access to the lockbox:

```
lbox access add <contact name>
```

You can share access to any number of contacts to the lockbox.

You can now safely email the lockbox to each contact.

