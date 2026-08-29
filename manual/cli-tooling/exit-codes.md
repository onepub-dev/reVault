# Exit Codes

The CLI tooling uses a standard set of exit codes to indicate different error conditions:



Scripts should use the CLI exit status instead of parsing human-readable error messages.&#x20;

The stable exit codes are&#x20;

* `0` for success
* `1` for an unclassified failure
* `2` for invalid command usage or input
* `10` when the lockbox session is closed,
* `11` when authentication fails
* `12` when an entry is not found
* `13` when the local vault is unavailable
* `14` for an unsupported lockbox or vault format
* `15` for corrupt or truncated data.&#x20;



In particular, exit code `10` means the caller should ask the user to run `lbx open <lockbox>` and retry.&#x20;

Error descriptions and recovery guidance are written to standard error and may change over time without changing these numeric codes.
