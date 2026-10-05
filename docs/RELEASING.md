# Releasing ISC MingoSolve

For the owner. A release is a tag `vX.Y.Z` on `main`; `.github/workflows/release.yml` does the rest: it builds the
installers on Linux (Ubuntu 22.04, so they run on older glibc), macOS (one universal `.dmg` for Apple Silicon and
Intel, ad-hoc signed) and Windows (NSIS), signs the updater bundles, attaches everything to a GitHub Release here,
mirrors it to the public repo `isc-fs/iskapps` (tag `mingosolve-vX.Y.Z`), writes `mingosolve/latest.json` there (the
URL the app's updater reads), and finally fast-forwards `dev` to `main`.

## One-time setup

1. **Create the updater key pair** (on your machine, never in the repo):
   ```bash
   mkdir -p ~/.tauri
   npx --yes @tauri-apps/cli@^2 signer generate -w ~/.tauri/mingosolve-updater.key
   ```
   Choose a password (or leave it empty). It writes `~/.tauri/mingosolve-updater.key` (private) and
   `~/.tauri/mingosolve-updater.key.pub` (public). Back the private key and its password up somewhere safe
   (password manager): if you lose it, installed copies can never be updated again and everybody reinstalls by hand.

2. **Repository secrets** (GitHub: isc-fs/MingoSolve → Settings → Secrets and variables → Actions → Secrets), or
   with the `gh` CLI:
   ```bash
   gh secret set TAURI_SIGNING_PRIVATE_KEY --repo isc-fs/MingoSolve < ~/.tauri/mingosolve-updater.key
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo isc-fs/MingoSolve   # paste the password; empty if none
   gh secret set ISKAPPS_TOKEN --repo isc-fs/MingoSolve                        # paste the token from step 3
   ```
   The password secret must exist even when the password is empty.

3. **`ISKAPPS_TOKEN`**: a fine-grained personal access token (GitHub → Settings → Developer settings → Personal
   access tokens → Fine-grained tokens) with **Resource owner** `isc-fs`, **Repository access: only** `isc-fs/iskapps`,
   and the single permission **Contents: Read and write**. Give it an expiry and put the renewal date in your
   calendar: an expired token makes the publish job fail (loudly) on release day.

4. **Repository variable** (same page, Variables tab; it is public information):
   ```bash
   gh variable set TAURI_UPDATER_PUBKEY --repo isc-fs/MingoSolve < ~/.tauri/mingosolve-updater.key.pub
   ```
   The workflow injects it into the build with a `--config` override, so `tauri.conf.json` keeps an empty `pubkey`
   and local `tauri build` needs no keys. Do not commit the key or set `createUpdaterArtifacts` in the file.

5. `isc-fs/iskapps` must exist, be public and have a `main` branch. The workflow commits only
   `mingosolve/latest.json` there.

6. If `dev` has branch protection that blocks direct pushes, the last job (`sync-dev`) fails after the release is
   already published. Either allow `github-actions[bot]` to push to `dev`, or fast-forward `dev` yourself afterwards
   (`git fetch && git switch dev && git merge --ff-only origin/main && git push`).

## Dry run (nothing is published)

Actions → **Release** → Run workflow, on `main` or any branch. GitHub only offers a manual run for a workflow that
exists on the default branch (`main`), so before the first release reaches `main` there is no dry run yet. It runs the three builds with signing on and
uploads the installers as workflow artefacts (`installers-ubuntu-22.04`, `installers-macos-latest`,
`installers-windows-latest`, kept 7 days). No release, no tag, no iskapps access. Do this after any change to the
workflow, to `tauri.conf.json` or to the icons, and once before the first real release. From a terminal:
```bash
gh workflow run release.yml --repo isc-fs/MingoSolve --ref main
gh run watch --repo isc-fs/MingoSolve
```
Install the macOS and Windows artefacts on a real machine before tagging.

## Cutting a release

1. **Bump the version** on a branch cut from `dev`, in all of these (they must be equal; the workflow checks them
   and `Cargo.lock`):
   - `Cargo.toml` (root, `version`)
   - `apps/mingosolve/src-tauri/Cargo.toml`
   - `apps/mingosolve/package.json` (and the matching entries in `package-lock.json`, via `npm install --package-lock-only`)
   - `apps/mingosolve/src-tauri/tauri.conf.json`

   Then `cargo check --workspace` so `Cargo.lock` follows. Update `ROADMAP`/`.github/roadmap.yaml` if the phase
   is done.
2. Open a PR **to `dev`**, let CI pass, merge.
3. Open a PR **`dev` → `main`**, merge it.
4. Tag `main` and push the tag (the only manual publishing step):
   ```bash
   git fetch origin && git switch main && git pull --ff-only
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
5. Watch the run (`gh run watch`). When it is green: a Release exists here, `isc-fs/iskapps` has a release
   `mingosolve-vX.Y.Z`, `mingosolve/latest.json` on its `main` shows the new version, and `dev` equals `main`.

If a build leg fails, fix the cause and re-run the failed jobs. If the tag itself was wrong (versions disagree),
delete it (`git push --delete origin vX.Y.Z`, `git tag -d vX.Y.Z`), fix, and tag again.

## Verify the updater

Do this with the previous release installed (so there is something to update from):

1. `curl -s https://raw.githubusercontent.com/isc-fs/iskapps/main/mingosolve/latest.json | jq .` shows the new
   `version`, and `linux-x86_64`, `windows-x86_64`, `darwin-aarch64`, `darwin-x86_64` each have a `signature` and a
   `url` under `https://github.com/isc-fs/iskapps/releases/download/mingosolve-vX.Y.Z/`.
2. Each `url` downloads (`curl -sIL <url> | head -1` gives 200).
3. Start the older installed app: the banner offers the new version; **Install** downloads, restarts, and Settings
   shows the new version. A signature error means `TAURI_UPDATER_PUBKEY` does not belong to
   `TAURI_SIGNING_PRIVATE_KEY`: the pair was changed after an earlier release was installed.

The app also has a switch to disable the startup check (Settings → Updates); a manual check always works.

## Quiz freeze

Before a quiz, publish nothing. The last release before the quiz is the one teammates run; they tick off automatic
update checks themselves (see [INSTALL.md](INSTALL.md)). A release during quiz week would reach every machine that
still has the check on.

## If the signing key leaks

An attacker with the private key and write access to the update URL could push a malicious update that installed
apps would accept. The update URL is on `isc-fs/iskapps`, which only your token (and org admins) can write to, so
act on both:

1. Revoke `ISKAPPS_TOKEN` and the leaked key's secrets in the repository settings; check the history of
   `isc-fs/iskapps` for commits to `mingosolve/latest.json` you did not make, and delete the suspicious releases.
2. Generate a new pair (setup step 1, a new file name), update `TAURI_SIGNING_PRIVATE_KEY`,
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and `TAURI_UPDATER_PUBKEY`, and issue a new release.
3. Installed apps still trust the **old** public key, so they cannot verify the new release and the in-app update
   fails. Everyone must reinstall once by hand from the releases page ([INSTALL.md](INSTALL.md)); tell the team
   through the usual channel and say to ignore any update banner until then. After the reinstall, updates work again
   with the new key.
