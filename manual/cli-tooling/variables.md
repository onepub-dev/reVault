# Variables

reVault lets you securely store variables in a Lockbox.

A variable is simple a name/value pair:

```
DB_PORT: 80
```

Storing variables in a Lockbox can be useful for CI/CD environments allowing to keep all of the data necessary for a CI/CD run in a single secure archive.

### Storing a variable

To add a variable to your lockbox run:

```
lockbox open mystuff.lbox
lockbox var set DB_PORT 80
lockbox var set DB_HOST 127.0.0.1
lockbox close mystuff.lbox
```

### Value source

When setting a variable you can take the value for the variable from one of a possible number of sources:

```
  -i, --interactive      Prompt for the value.
  -t, --stdin            Read the value from stdin.
  -v, --value <VALUE>    Read the value from this argument.
  -f, --file <FILE>      Read the value from a file.
  -e, --from-env <NAME>  Read the value from a process environment variable.

```

If you don't proved a source switch then the command assumes that the value argument is a positional argument. The `--value` switch is provided for symmetry.

### Displaying a variable

```
lockbox open mystuff.lbox
lockbox var get mystuff.lbox DB_PORT
lockbox var get mystuff.lbox DB_HOST
lockbox close mystuff.lbox
```

## Secrets

reVault provides a second layer of security for variables that contain sensitive information such as an API\_KEY.

Variables that are marked as secret are never written to disk in clear text and are stored in locked OS memory so that they cannot be written to the page file or core dumped (where supported).

See the [reVault Session Agent](revault-session-agent.md) for additional details on how reVault protects your secrets.



To store and display secret variables we pass the --secret flag.

Setting a secret has additional restrictions to setting a normal variable.  When setting a secret you cannot pass the value as a command line argument nor use the `--value` switch. This is done to help protect your secrets as command line arguments are visible via the process list (e.g. ps -A).

Instead, use one of the alternate input value mechanisms

```
 -i, --interactive      Prompt for the value.
  -t, --stdin            Read the value from stdin.
  -f, --file <FILE>      Read the value from a file.
  -e, --from-env <NAME>  Read the value from a process environment variable.
```

```
lockbox open mystuff.lbox
lockbox var set --secret --interactive mystuff.lbox API_KEY 
> Secret value: 
lockbox var get --secret mystuff.lbox API_KEY
lockbox close mystuff.lbox
```

### Paths

A lockbox variable can be stored under  a path:

```
lockbox open mystuff.lbox
lockbox var set mystuff.lbox /accounting/production/DB_PORT 80
lockbox var set mystuff.lbox /accounting/staging/DB_PORT 81
lockbox var get mystuff.lbox /accounting/production/DB_PORT
lockbox close mystuff.lbox
```

### List

You can list all of the variables stored in a lockbox as well as their sensitivity:

```
lockbox var list mystuff.lbox 
name              sensitivity
/APIO_KEY         normal     
/product/API_KEY  secret 
```

You can pass a glob to the list command to filter the output:

```
lockbox var list secrets.lbox '**/API_KEY'
name              sensitivity
/API_KEY          normal     
/product/API_KEY  secret   
```

### Export

The export command is designed to extract the name/value pairs of variables from a lockbox and potentially expose them as OS environment variables.
