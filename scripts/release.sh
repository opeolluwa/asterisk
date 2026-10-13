#!/usr/bin/env bash
set -e

CARGO_TOML="Cargo.toml"

BUMP_TYPE=${1:-patch}

CURRENT_VERSION=$(awk -F'"' '/^version = / {print $2; exit}' "$CARGO_TOML")

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

case "$BUMP_TYPE" in
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  patch)
    PATCH=$((PATCH + 1))
    ;;
  *)
    echo "Invalid bump type: $BUMP_TYPE"
    echo "Use: patch | minor | major"
    exit 1
    ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
TAG="v$NEW_VERSION"

echo "Current version: $CURRENT_VERSION"
echo "New version: $NEW_VERSION"

tmp=$(mktemp)
sed "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML" > "$tmp"
mv "$tmp" "$CARGO_TOML"

cargo metadata --no-deps >/dev/null

git add Cargo.toml Cargo.lock
git commit -m "release: v$NEW_VERSION"

git tag "$TAG"

git push origin "$TAG"

echo "Release $TAG created"