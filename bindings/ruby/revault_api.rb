# frozen_string_literal: true

# Encrypt files, variables, and typed form records in portable reVault
# lockboxes, and manage keys and local vault metadata.
#
# {Revault::Vault} is the main entry point. Release owned handles promptly and
# use callback-scoped secret accessors to avoid retaining plaintext. See the
# {repository README}[https://github.com/onepub-dev/reVault#readme] for
# installation, security guidance, and complete examples.

require_relative 'lib/revault/vault'
