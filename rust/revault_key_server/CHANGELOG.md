# Changelog

## Unreleased

- Enforce configured public request and verification-email limits using the
  socket peer address. Proxies must enforce their own client-aware limits.
- Deliver email verification links through email only; publisher responses no
  longer expose the bearer link. Restart all old processes to invalidate pending
  links and review existing verification claims separately.
- Add replicated reciprocal exchange with terminal receipts and explicit
  single-owner promotion. `fencing-help` explains how to fence the old owner
  before promoting a replacement.
