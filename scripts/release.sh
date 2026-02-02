#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting release process...${NC}"

# 1. Check if we are on main branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "main" ]; then
    echo -e "${RED}Error: You must be on the main branch to release.${NC}"
    exit 1
fi

# 2. Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: You have uncommitted changes. Please commit or stash them first.${NC}"
    exit 1
fi

# 3. Get version from Cargo.toml
VERSION=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
TAG="v$VERSION"
echo -e "Detected version from Cargo.toml: ${GREEN}$VERSION${NC}"

# 4. Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag $TAG already exists locally.${NC}"
    exit 1
fi

if git ls-remote --tags origin "$TAG" | grep -q "$TAG"; then
    echo -e "${RED}Error: Tag $TAG already exists on remote.${NC}"
    exit 1
fi

# 5. Run quality checks
echo -e "${YELLOW}Running tests and checks...${NC}"
cargo fmt -- --check
cargo clippy -- -D warnings
# Note: Regression tests are usually run via cargo run -- test, but here we run standard unit tests
cargo test

# 6. Verify version in CHANGELOG.md (optional but recommended)
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo -e "${RED}Error: Version $VERSION not found in CHANGELOG.md. Did you forget to update it?${NC}"
    exit 1
fi

# 7. Tag the current commit
echo -e "${YELLOW}Tagging version $TAG...${NC}"
git tag -a "$TAG" -m "Release $TAG"

# 8. Push to GitHub
echo -e "${YELLOW}Pushing to origin...${NC}"
git push origin main
git push origin "$TAG"

echo -e "${GREEN}Release $TAG successfully pushed to GitHub!${NC}"
echo -e "GitHub Actions should start building the release shortly."
