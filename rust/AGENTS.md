When the vault or lockbox format changes we MUST create and publish a new set of migration scripts that upgrade the vault and/or lockboxes as appropriate.

When prepareing for a release we need to ensure that the set of language bindings have been updated to reflect any modifications to the apis.

Our aim is a pure rust implementation so dependencies and transient dependencies that contain C code or other unsafe languages should not be permitted.

Any script tools must be written rust.