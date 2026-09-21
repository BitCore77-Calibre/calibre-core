#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

BIN=./target/release/solochain-template-node

echo "=== 1. Back up website ==="
rm -rf /tmp/calibre-website
cp -r website /tmp/calibre-website

echo "=== 2. Go to main ==="
git checkout main

echo "=== 3. Delete stale local gh-pages ==="
git branch -D gh-pages 2>/dev/null || true

echo "=== 4. Create orphan gh-pages ==="
git checkout --orphan gh-pages

echo "=== 5. Remove everything tracked ==="
git rm -rf . >/dev/null 2>&1 || true

echo "=== 6. Remove every untracked file ==="
git clean -fdx >/dev/null 2>&1 || true

echo "=== 7. Copy ONLY website files ==="
cp /tmp/calibre-website/index.html .
cp -r /tmp/calibre-website/assets .
touch .nojekyll

echo "=== 8. Contents (must be: assets index.html .nojekyll) ==="
ls -la

echo "=== 9. Commit ==="
git add -A
git commit -m "deploy: Calibre Protocol landing page"

echo "=== 10. Force-push ==="
git push --force origin gh-pages

echo "=== 11. Return to main ==="
git checkout main

echo "=== DONE ==="
