#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

BIN=./target/release/solochain-template-node

echo "=== 1. Back up website files to /tmp ==="
rm -rf /tmp/calibre-website
cp -r website /tmp/calibre-website

echo "=== 2. Switch to main ==="
git checkout main

echo "=== 3. Delete stale local gh-pages ==="
git branch -D gh-pages 2>/dev/null || true

echo "=== 4. Create orphan gh-pages in a SEPARATE worktree ==="
# Instead of mutating the current repo, use a temp worktree
TMPWT=/tmp/calibre-ghpages-worktree
rm -rf "$TMPWT"
git worktree add --detach "$TMPWT" 2>/dev/null || true
cd "$TMPWT"
git checkout --orphan gh-pages 2>/dev/null || true
git rm -rf . >/dev/null 2>&1 || true

echo "=== 5. Copy website files ==="
cp /tmp/calibre-website/index.html .
cp /tmp/calibre-website/mempool.html . 2>/dev/null || true
cp -r /tmp/calibre-website/assets .
touch .nojekyll

echo "=== 6. Contents ==="
ls -la

echo "=== 7. Commit ==="
git add -A
git commit -m "deploy: Calibre Protocol landing page + mempool"

echo "=== 8. Force-push to remote gh-pages ==="
git push --force origin gh-pages

echo "=== 9. Clean up worktree ==="
cd /
git worktree remove --force "$TMPWT" 2>/dev/null || rm -rf "$TMPWT"

echo "=== DONE ==="
