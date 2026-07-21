# macOS local signing (making permissions stick)

## The problem

On macOS, the app needs two TCC permissions:

- **Microphone** — to record audio.
- **Accessibility** — for the global hotkey (a CGEvent tap) and auto-paste
  (synthetic key events).

macOS ties these grants to the app's **code-signing identity**. Our CI /
GitHub releases are only **ad-hoc signed**, which means the code identity
(cdhash) changes on *every* build. So each time you update the app, macOS sees
a "different" app, silently invalidates the old Accessibility grant, and the
hotkey stops working — the app still appears (toggled on!) in
System Settings ▸ Privacy & Security ▸ Accessibility, but it isn't actually
trusted. Re-toggling usually doesn't fix it.

The fix for local use is to sign every local build with **one stable,
self-signed certificate**. That gives a fixed *Designated Requirement*, so you
grant Accessibility once and it persists across rebuilds.

> This does not help apps installed from GitHub releases (still ad-hoc). For a
> distribution-grade fix, the release workflow needs a Developer ID certificate
> + notarization (see the TODO in `.github/workflows/release.yml`). Until then,
> build locally with the script below.

## One-time setup: create the signing certificate

Creates a self-signed code-signing cert named **"Speech AI Tool Local Signing"**
in your login keychain. Run once.

```bash
# 1. Generate a self-signed cert with the Code Signing extended key usage.
CN="Speech AI Tool Local Signing"
openssl genrsa -out /tmp/sat.key 2048
openssl req -x509 -new -key /tmp/sat.key -out /tmp/sat.crt -days 3650 \
  -subj "/CN=$CN" \
  -addext "basicConstraints=critical,CA:false" \
  -addext "extendedKeyUsage=critical,codeSigning" \
  -addext "keyUsage=critical,digitalSignature"

# 2. Package as PKCS#12 using LEGACY algorithms (Apple's importer can't read
#    OpenSSL 3's default MAC).
openssl pkcs12 -export -out /tmp/sat.p12 -inkey /tmp/sat.key -in /tmp/sat.crt \
  -passout pass:sat -name "$CN" \
  -legacy -certpbe PBE-SHA1-3DES -keypbe PBE-SHA1-3DES -macalg sha1

# 3. Import into the login keychain. -A lets codesign use the key without a
#    per-signature prompt.
security import /tmp/sat.p12 -k "$HOME/Library/Keychains/login.keychain-db" \
  -P sat -T /usr/bin/codesign -A

# 4. Clean up the temporary key material.
rm -f /tmp/sat.key /tmp/sat.crt /tmp/sat.p12
```

The certificate shows as `CSSMERR_TP_NOT_TRUSTED` in
`security find-identity` — that's fine. Trust affects *verification*, not
*signing*, and macOS runs a non-quarantined self-signed app without it.

**First build only:** the first time `codesign` uses the key, macOS shows a
keychain dialog ("codesign wants to sign using key ... in your keychain").
Click **Always Allow** (enter your login password) so later builds are silent.

## Build + sign + install

```bash
scripts/build-local-macos.sh --install
```

This builds the release app, signs it with the stable identity, verifies the
signature, and copies it to `/Applications`. Omit `--install` to just build.

## One-time setup: grant Accessibility

After the first signed install:

1. Launch **Speech AI Tool**.
2. Open **System Settings ▸ Privacy & Security ▸ Accessibility**.
3. If a stale entry exists, remove it (or run
   `tccutil reset Accessibility com.speech-ai-tool.app`), then add / enable
   `/Applications/Speech AI Tool.app`.
4. The global hotkey now works and **keeps working across future local
   rebuilds**, because they share the same signing identity.

Verify from the terminal:

```bash
tccutil reset Accessibility com.speech-ai-tool.app   # only if starting clean
codesign -d -r- "/Applications/Speech AI Tool.app" | grep designated
# => designated => identifier "com.speech-ai-tool.app" and certificate leaf = H"..."
```

The certificate-leaf hash is what stays constant across rebuilds.
