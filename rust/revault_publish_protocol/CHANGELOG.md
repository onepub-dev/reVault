# Changelog

## Unreleased

- Bound plain HTTP response headers and bodies during reading and enforce a
  total read deadline, including slowly delivered responses.
- Add signed reciprocal exchange offers, acceptance and independent fingerprint
  comparison with bounded HTTPS transport.
