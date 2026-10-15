# Reporting a vulnerability

Please report anything security-relevant through
[GitHub's private advisory form](https://github.com/wormholesystems/wormholesystems/security/advisories/new)
rather than a public issue.

## What is worth reporting

A deployment holds the EVE refresh token of every linked character, so anything that reads
another account's data, escalates a role on a map, or gets at the database is worth
raising. So is anything that lets a map be read or written by somebody the access rules say
should not be able to.

## What is already known

- **Refresh tokens are stored unencrypted.** Whoever can read the database can act as any
  linked character within the scopes it granted. This is a known property rather than a
  bug; see the note in `migrations/0001_auth.sql`.
- **This is beta.** It has not been audited. Please report things anyway, knowing early is
  the point of saying so.
