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

# 3. Get current version
CURRENT_VERSION=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "Current version: ${GREEN}$CURRENT_VERSION${NC}"

# 4. Ask for new version
read -p "Enter new version (e.g., 0.6.1): " NEW_VERSION
if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}Error: Version cannot be empty.${NC}"
    exit 1
fi

# 5. Run quality checks
echo -e "${YELLOW}Running tests and checks...${NC}"
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test

# 6. Update Cargo.toml
echo -e "${YELLOW}Updating Cargo.toml to $NEW_VERSION...${NC}"
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
cargo check # Update Cargo.lock

# 7. Update CHANGELOG.md
echo -e "${YELLOW}Updating CHANGELOG.md...${NC}"
DATE=$(date +%Y-%m-%d)
# Replace [Unreleased] with [NEW_VERSION] - DATE
sed -i "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[$NEW_VERSION\] - $DATE/" CHANGELOG.md

# 8. Commit and Tag
echo -e "${YELLOW}Committing and tagging...${NC}"
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: Bump version to $NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

# 9. Push to GitHub
echo -e "${YELLOW}Pushing to origin...${NC}"
git push origin main
git push origin "v$NEW_VERSION"

echo -e "${GREEN}Release v$NEW_VERSION successfully pushed to GitHub!${NC}"
echo -e "GitHub Actions should start building the release shortly."
