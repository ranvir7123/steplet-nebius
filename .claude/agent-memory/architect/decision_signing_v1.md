---
name: decision-signing-v1
description: 2026-10-06 decision on runfile sender identity and signing - SSHSIG (OpenSSH signature format) with Ed25519, embedded in metadata, textual canonical form, TOFU pins plus GitHub key lookup for verified tier
metadata:
  type: project
---

**Decided 2026-10-06.** Runfiles are signed with the OpenSSH signature format (SSHSIG, PROTOCOL.sshsig), Ed25519 keys only, namespace `runfile-v1`, hash sha512.
Signature is an armored block under `metadata.runfile-signature: |` and must be the last thing before the closing `---`.
Signed bytes = whole file after a textual canonical form (strict UTF-8, drop BOM, CRLF/CR to LF, strip trailing space/tab per line, exactly one final LF) with the signature block lines removed. No YAML re-serialisation. Parse YAML once, from the signed bytes only; reject duplicate keys, anchors, aliases and tags.
Trust: local pins keyed by key fingerprint (trust on first use). Verified tier = key appears on the claimed GitHub account (`api.github.com/users/<u>/ssh_signing_keys` or `github.com/<u>.keys`), pinned by numeric GitHub id because usernames get recycled.

**Rejected:** do-nothing (cannot say who sent it); minisign (no GitHub key tie-in, trusted comment redundant, thin JS support); own Ed25519 envelope (runner-up, same crypto but loses stock `ssh-keygen` signing and checking); Sigstore keyless (network to sign, GitHub login yields email not username, signer email published forever in a public log, sigstore-js Node-only, sigstore-rs pre-1.0); OpenPGP cleartext (dash-escaping turns the leading `---` into `- ---`, breaking frontmatter; nobody has keys); JWS (no key distribution story, JSON-shaped).

**Why:** same Ed25519 cryptography as the simplest option, but developers' existing keys, `ssh-keygen`, and GitHub's key lists work unchanged. Prototype on 2026-10-06 showed WebCrypto-made signatures verify with Windows' built-in ssh-keygen 9.5.

**Flip condition:** if the verified tier must name non-developers (no GitHub, no SSH key), revisit Sigstore or an email-based scheme.

**How to apply:** do not re-open format choice without a new constraint. See [[constraint-skillmd-compat]].
