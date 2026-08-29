# reVault

reVault is an modern archive format (know as a lockbox) that encrypts, compresses and signs each archive.

The core engine is written in Rust and is designed to be fast and use a small memory footprint.&#x20;

As well as the core engine, reVault provides [CLI](cli-tooling/) tooling to create lockboxes and [16+ language  bindings](apis/revault-api.md) that makes it easy to create/managed lockboxes from any supported language.&#x20;

If you are just want to take reVault for a quick spin go to the [quick start guide](cli-tooling/quick-start-guide.md).

The reVault git repo can be found [here](https://github.com/onepub-dev/reVault).

reVault is a re-imagining of the classic archive format but hopefully better.&#x20;

reVault stores:

* entire directory structures including files, file permissions and symlinks
* variables stored under a path structure:  /production/API\_KEY, /staging/API\_KEY
* form data with an basic interactive form editor (e.g name, url, username and password)

A reVault archive is referred to as a Lockbox.&#x20;

A Lockbox is:

* compressed
* encrypted&#x20;
* signed

reVault is designed to:

* be intuitive to use
* be secure by default
* allow fast random access (read/write) to files and data stored in the lockbox
* be recoverable - even if large sections of a lockbox is corrupted you can still recover uncorrupted sections.
* cross platform with support for Linux, MacOS, Windows, x64, ARM and Wasm.

Sharing:

* lockboxes are designed to be shared.
* lockboxes are opened using a key (or password) and can be opened with different keys. You can add the keys of an associated to a lockbox so that associates can also open the lockbox.
* create any number of lockboxes and share each with a different group of associates.
* the lockbox file format is cross platform including including arm and web.

## reVault concepts

reVault uses a Vault and Lockboxes.

### Vault

The vault is used to store you master key(s). Each of your associates will have their own vault and master keys.&#x20;

The vault can also store a contact list of your associate's keys so once an associated has shared their key with you can easily add them as a recipient of your lockbox.

The vault also backups up the set of keys/passwords used by your lockbox to aid in the recover of a corrupt lockbox.

The contents of the vault are stored in a lockbox to ensure that your keys are secure.

### Lockboxes

A Lockbox is the archive file - like a zip file but highly secure.

You have a single Vault (which you need to securely backup) but any number of Lockboxes.

A lockbox can store:

* files/directories
* symlinks
* file permissions
* variables
* forms

### Variables

A Lockbox allows you to store name/value pairs which we refer to as a variable:

e.g.

API\_KEY: XXXXXXXXXXX

You can add a path to the key so you can use the same name multiple times:

`/production/API_KEY: XXXXXXX`

`` /staging/API_KEY: XXXXXX` ``

`You can then set/get those values using the Lockbox command line tools.`

### Forms

Forms are very similar to variables but are intended to group related items.

For example you might need to store the details about a web site such as:

* name/description
* url
* username
* password

You can create a (reusable) form that allows you to capture those fields and store them under the provided name. &#x20;

Like variables, you can get/set the values of a form field.

Unlike variables, Form fields have a type which we validate against when you attempt to enter a value into a form.

The available types are: text, secret, password, url, email, date, month, notes, number

### Secrets

The whole point of a Lockbox is to keep your data secure, but not all data is equal and some data needs to be treated with a greater degree of security.

reVault lets you define specific variables and form fields as 'secret'.

As an example you may (should) define an variable that stores an API key as 'secret'.

Secrets in reVault are treated very careful with considerable effort taken to ensure they are never placed where are third party could get access to them. &#x20;

Refer to the technical section on [Secrets](cli-tooling/secrets.md) to learn more about how we manage them.

## Session Agent

The Session Agent is a hidden process/program that runs in the background and is started when you run the first lockbox command in an OS login session.

The Agent's job is to make interacting with lockbox easier by reducing how often you need to enter your password (it keeps a copy safe for you) and helps to manage secrets. &#x20;

The agent can also automatically open your vault when you login to your desktop session, in this mode you never need to enter a password when accessing your vault or a lockbox but your data is still kept safe.

The Agent is available on Linux, MacOS and Windows.

The agent also allows you to set an 'active' lockbox which removes the need to specify a lockbox for every command.&#x20;

## Getting started

Install the lockbox cli using cargo:

cargo install revault\_cli

The install process will add three CLI commands to your path:

* `lockbox` - the main lockbox cli tool used to create lockboxes.
* lbx - an short hand for lockbox.
* lockbox\_agent - the lockbox session agent which the lockbox command starts as necessary.

### Initialise your Vault

Once installed you need to do a one time initialisation of the Vault:

```bash
lbx vault init
```

Your master key has now been created.

You should now backup your vault by running:

```bash
lbx vault backup
```

The backup will provide details on how to manage the resulting output.

!!!! BACKUP IS CRITICAL - the backups need to be stored on a separate secure storage device in case your desktop disk fails. Loss of your vault and backup will mean loss of access to ALL of your LOCKBOXES !!!!!

You can run the doctor command to get details about your install:

```bash
lbx doctor
```

### Create a Lockbox

You are now ready to create your first lockbox.

All lockboxes use the .lbox extension.

Create a lockbox:

```bash
lbx create secrets.lbox
```

Open your lockbox so you can add files/data to the lockbox:

```bash
lbx open secrets.lbox
```

Close the lockbox when you are done:

```bash
lbx close secrets.lbox
```

If you forget to the lock the box, don't panic, the Agent will automatically lock the box for you after a short period of time (usually 30 min).

Add a directory and the files contained in it:

```bash
lbx add secrets.lbox ./project
```

Add a directory, its files and all subdirectories and their files.

```bash
lbx add secrets.lbox --recursive ./project
```

Add a file to the lockbox:

```bash
lockbox add secrets.lbox readme.md 
```

Add a file and place under the `/docs` directory within the lockbox:

```bash
lbx add secrets.lbox readme.md /docs
```

List and extract:

```bash
lbx ls secrets.lbox /
lbx ls secrets.lbox /project '**/*.rs'
lbx extract secrets.lbox /project/README.md ./out/README.md
```

Delete a file or directory from the lockbox:

```bash
lockbox rm secrets.lbox readme.md    
```

## Auto Open&#x20;

When you first run 'lbx vault init\`, reVault will default to 'auto-open'.

Auto-open is dependant on your platform supporting the [Session Agent.](./#session-agent)

With auto-open enabled, you don't need to 'open' a lockbox to work on it.

Instead of:

```
lbx open secrets.lbox
lbx add secrets.lbox readme.md
```

You can simply do:

```
lbx add secrets.lbox readme.md
```

You can then manually close the lockbox, but the session agent (where supported) will automatically close the lockbox after 30 minutes.

You can disable auto-open via the  `lbx vault auto-open off` command.

## Active lockbox

You can further abbreviate lockbox commands by setting a single lockbox as active:

```
lbox session ativate secrets.lbox
```

Once you have activated a lockbox instead of:

```
lbox add secrets.lbox readme.md
```

You can simply write:

```
lbox add readme.md
```

You can deactive the active lockbox using the deactive command:

```
lbox session deacivate
```

## Keys And Sharing

### Your master key

When you create a lockbox it is locked using you default master key (referred to as an Identity) in your vault (or a classic password if you choose).

Your master key is made up of two parts - a private part (often referred to as a private key) and a public part (or public key).  Going forward we will use the terms private key and public key as these are the most commonly used terms - but remember that these are just two parts of your master key.

The public part (public key) of your master key is used to close a lockbox. &#x20;

The private part (private key) is used to open a lockbox.

Think about it this way - you're probably happy for anyone to close your lockbox but you are the only one that should be able to open your lockbox - so you need to keep the opening key 'private', hence why it is called a private key.

As its name suggests the public key (part) can be made public and shared with anyone without risk of compromising your security or the security of your lockboxes.

As the private key (part) is used to open a lock box - the private key MUST NEVER BE SHARED.

You don't normally need to worry about your private key as reVault manages it for you, but a user that has access to your desktop, whilst you are logged in, may be able to access your private key ( so lock your desktop before you walk away from your desk).

### Sharing

Every other user of reVault also has their own master key and of course that means they have a public and private key (the two parts of the master key).

To share a lockbox with another reVault user we need to ask them for their public key.

Your associate has a number of ways of providing their public key. The easiest way is to ask them to publish their public key (public keys by their name can be made public without concern).

Get your associate to run the following command on their desktop:

```bash
lbx vault publish
```

The publish command will print out a share code and and a fingerprint.

NOTE: do not accept a share code nor fingerprint from an associated that contacted you - instead you must always be the one to initiate the communication channel (e.g. you call them) this is to avoid being fooled by an AI impersonating your associate.

Ask your associate for the share code and run:

```bash
lbx vault receive <share-code> [<contact name>]
```

Your associates public key has now been added to your vault's contact list under their contact name. To see the entry run:

```bash
lbx vault contact list
```

You can now add your associate to a lockbox:

```bash
lbx access secrets.lbox add "<contact name>"
```

It is now safe to email the lockbox to your associated and they will be able to open the lockbox.

Now you have the associated in your contact list you can add them to any lockbox and create lockboxes just for them (you will also have access).

```bash
lbx create myrecipes.lbox --for "<contact name>"
lbx add myrecipes.lbox ./recipies
```

Vault contact names are purely local. They are not copied into a lockbox when you give a contact access. Lockboxes do not contain contact names or email addresses as membership metadata, to avoid leakage when sharing a lockbox with multiple associates.

## Variables

reVault lets you store name/value pairs in a lockbox along with files and forms (or just by themselves). We refer to these name/value pairs as variables.

A name/value is just that a name associated with a value.

e.g. API\_KEY: XXXXXX

You can access an variable  (var) via its name to get or set its value.

### Plain values

Plain variables are for values that contain useful data, like configuration data, but not high-value secrets:

Here we show an example of 'setting' (or storing) an variable called 'DATABASE\_URL' with a value 'postgres://localhost/app'. We then we retrieve the value using the 'get' method.

```bash
lbx var set secrets.lbox DATABASE_URL 'postgres://localhost/app'
lbx var get secrets.lbox DATABASE_URL
```

### Secret values

Secret variables are for passwords, API tokens, signing keys, and similar critical material - generally data that gives access to other data.

reVault handles secret values vary careful, however there is a cost to doing so. Secret values are stored in 'locked memory' which if used excessively can cause your machine to run out of memory. For typical variables and form fields this isn't an issue and having hundreds of them in a lockbox isn't a problem. Just don't go trying to store megabytes of data in a secret variable or a secret form field!

To set a secret variable, the value must be provided through an explicit source:

```bash
lbx var set secrets.lbox --secret API_TOKEN --interactive
lbx var set secrets.lbox --secret API_TOKEN --file ./api-token.txt
lbx var set secrets.lbox --secret API_TOKEN --stdin
lbx var set secrets.lbox --secret API_TOKEN --from-env API_TOKEN
```

Avoid passing secrets as command-line arguments. Shell history and process inspection can expose those values. Prefer `--interactive`, `--stdin`, `--file`, or `--from-env`.

Sensitivity is chosen when a variable is created. Updating the value preserves that sensitivity. To change a plain variable to secret, or secret to plain, delete and recreate the variable.

## Avoiding Data Leakage

Protect your data is important so here are some tips to avoid your data being exposed to undesirable elements.&#x20;

NOTE: by default reVault enables auto-open which means that you don't need to open a lockbox or your vault. In highly secure environments you should disable auto-open by running 'lbx session auto-open off'.

Use these defaults unless you have a specific reason not to:

* Use `lockbox open` and the session agent instead of repeatedly entering your password.
* Use secret variables or secret form fields for credentials and tokens.
* Do not pass secret values on the command line.
* Keep private key backups/exports offline, short-lived, and permission-restricted.
* Prefer contact keys over shared passwords for team access.
* Run `lockbox close secrets.lbox` when you have finished working with a lockbox. (if supported the session agent will close a lockbox after 30 min).
* Use `lockbox visualize secrets.lbox` for diagnostics; it intentionally avoids printing file paths, file contents, variable names, or variable values.

Lockbox protects data inside the container. It cannot protect a secret after you write it to a normal terminal, shell history, clipboard, exported file, build log, or process environment controlled by other tooling.

## reVault API access

reVault also ships a stand alone API called `revault_core` which allows you to create and maintain vaults.

You can read more about the API here.



## Documentation

* [CLI how-to](docs/cli_how_to.md): command examples.
* [Key management](docs/key_management.md): vaults, keys, recipients, and unlock caching.
* [Security audit](docs/security_audit.md): security analysis and release blockers.
* [File formats](docs/file_formats.md): lockbox/vault format details and page layout.
* [Implementation overview](docs/implementation_overview.md): architecture, page cache, recovery, browser access, and development notes.
* [Rust development](docs/rust_development.md): build, test, and API docs.
* [Terminology](docs/terminology.md): canonical naming.

## Development

Run the Rust checks:

```bash
cd rust
cargo fmt --all
cargo check --workspace --all-targets --all-features
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Generate Rust API docs:

```bash
cd rust
tools/generate_api_docs.sh
```

## License

Dvault Source Available License 1.0 - see [LICENSE](https://github.com/onepub-dev/reVault/blob/master/LICENSE/README.md).

The source remains available for inspection, modification, and redistribution. Derivative works must publish their corresponding source code in a publicly accessible repository such as GitHub or a similar service. The license also restricts third parties from offering dvault, modified dvault, or substantially similar dvault-derived functionality as a hosted, managed, or network-accessible service without a separate written license.
