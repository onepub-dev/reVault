# Changelog

## Unreleased

- Add durable, anonymous local completion accounting for reciprocal exchanges
  and `GET /v2/exchange/status`. Count both-party delivery once across retries,
  restarts and receipt expiry, with an explicit baseline. Preserve the existing
  exchange and legacy status protocols; replica imports are not local completions.
  Exchange directories now use an exclusive process lock; stop the local service
  before running offline `resync-peer` against the same state directory.

- Enforce configured public request and verification-email limits using the
  socket peer address. Proxies must enforce their own client-aware limits.
- Deliver email verification links through email only; publisher responses no
  longer expose the bearer link. Restart all old processes to invalidate pending
  links and review existing verification claims separately.
- Add replicated reciprocal exchange with terminal receipts and explicit
  single-owner promotion. `fencing-help` explains how to fence the old owner
  before promoting a replacement.
