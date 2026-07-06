#!/usr/bin/env bash
# Build the playground and publish it to the `gh-pages` branch.
#
# Validators (apps/playground/src/csa-validators.ts) are committed, so the
# hosted Test button works. Complete solutions live only in the git-ignored
# csa-solutions.local.ts (Solve button, dev-only). This build temporarily
# moves that file aside so solutions never enter the published bundle, then
# restores it — regardless of outcome.
#
# Prereqs: Node 22 (nvm), pnpm, a built Rust toolchain (for build:wasm), and
# push access to the `origin` remote.
set -euo pipefail

ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$ROOT"

SOLUTIONS="apps/playground/src/csa-solutions.local.ts"
DIST="apps/playground/dist"
BASE="${VITE_BASE:-/caturra/}"
REMOTE="$(git remote get-url origin)"

# 1. Hide the local solutions so vite can't bundle them; always restore.
HIDDEN=""
if [ -f "$SOLUTIONS" ]; then
  HIDDEN="$(mktemp)"
  mv "$SOLUTIONS" "$HIDDEN"
  trap 'mv "$HIDDEN" "$SOLUTIONS"' EXIT
fi

# 2. Build the WASM engine and the site under the Pages base path.
pnpm build:wasm
VITE_BASE="$BASE" pnpm --filter @caturra/playground build

# 3. Guard: fail loudly if any solution answer slipped into the bundle.
if grep -rqIl "export const SOLUTIONS" "$DIST/assets" 2>/dev/null; then
  echo "error: SOLUTIONS found in the built bundle — aborting deploy" >&2
  exit 1
fi

# 4. Publish dist/ to gh-pages (force — it is a build-artifact branch).
touch "$DIST/.nojekyll"
STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
(
  cd "$DIST"
  rm -rf .git
  git init -q
  git checkout -q -b gh-pages
  git add -A
  git -c user.name=deploy -c user.email=deploy@localhost commit -q -m "Deploy playground $STAMP"
  git push -f "$REMOTE" gh-pages
  rm -rf .git
)

echo
echo "Deployed to gh-pages ($STAMP)."
echo "Enable once in GitHub: Settings > Pages > Source: Deploy from a branch > gh-pages / (root)."
echo "Site: https://wilkie.github.io${BASE}"
