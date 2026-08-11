#!/usr/bin/env bash
# Changelog generation script
# Validates: Requirements 8.6, 8.7, 8.8

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo " Generating changelog..."

# Get current tag (if any)
CURRENT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

# Get previous tag
if [ -n "$CURRENT_TAG" ]; then
    PREV_TAG=$(git describe --tags --abbrev=0 "$CURRENT_TAG^" 2>/dev/null || echo "")
    echo -e "${GREEN} Current tag: $CURRENT_TAG${NC}"
    if [ -n "$PREV_TAG" ]; then
        echo -e "${GREEN} Previous tag: $PREV_TAG${NC}"
        RANGE="$PREV_TAG..$CURRENT_TAG"
    else
        echo -e "${YELLOW}  No previous tag found, using all commits${NC}"
        RANGE=""
    fi
else
    echo -e "${YELLOW}  No current tag found, using all commits${NC}"
    RANGE=""
fi

# Create changelog file
CHANGELOG_FILE="CHANGELOG.md"
echo "# Changelog" > "$CHANGELOG_FILE"
echo "" >> "$CHANGELOG_FILE"

# Extract version from Cargo.toml
VERSION=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
if [ -n "$VERSION" ]; then
    echo "## Version $VERSION" >> "$CHANGELOG_FILE"
    if [ -n "$CURRENT_TAG" ]; then
        echo "**Tag:** $CURRENT_TAG" >> "$CHANGELOG_FILE"
    fi
    echo "" >> "$CHANGELOG_FILE"
fi

# Get commit count
if [ -n "$RANGE" ]; then
    COMMIT_COUNT=$(git log --oneline "$RANGE" --no-merges | wc -l)
else
    COMMIT_COUNT=$(git log --oneline --no-merges | wc -l)
fi

echo "**Total commits:** $COMMIT_COUNT" >> "$CHANGELOG_FILE"
echo "" >> "$CHANGELOG_FILE"

# Function to extract PR number from commit message
extract_pr_number() {
    local msg="$1"
    local pr_pattern='(close[sd]?|fix(es)?|resolve[sd]?) #([0-9]+)'
    # Look for patterns like (#123) or closes #123
    if [[ "$msg" =~ \(#([0-9]+)\) ]]; then
        echo "${BASH_REMATCH[1]}"
    elif [[ "$msg" =~ $pr_pattern ]]; then
        echo "${BASH_REMATCH[3]}"
    fi
}

# Function to format commit message
format_commit() {
    local hash="$1"
    local subject="$2"
    local author="$3"
    local date="$4"
    
    # Remove type prefix for display
    local display_subject="$subject"
    if [[ "$subject" =~ ^[a-z]+(\([^)]+\))?:\ (.+) ]]; then
        display_subject="${BASH_REMATCH[2]}"
    fi
    
    # Extract PR number
    local pr_number=$(extract_pr_number "$subject")
    
    # Format entry
    local entry="- $display_subject"
    if [ -n "$pr_number" ]; then
        entry="$entry ([#$pr_number](https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/pull/$pr_number))"
    fi
    entry="$entry - $author"
    
    echo "$entry"
}

# Collect breaking changes
BREAKING_CHANGES=()

# Function to check if commit contains breaking changes
is_breaking_change() {
    local subject="$1"
    local body="$2"
    
    # Check for "BREAKING CHANGE:" in commit body
    if [[ "$body" =~ BREAKING[[:space:]]*CHANGE: ]]; then
        return 0
    fi
    
    # Check for "!" after type in subject (conventional commits)
    if [[ "$subject" =~ ^[a-z]+(\([^)]+\))?!:\ ]]; then
        return 0
    fi
    
    return 1
}

# Collect commits by type
declare -A commits_by_type
declare -A authors_by_type

# Commit types to track (from conventional commits)
TYPES=("feat" "fix" "docs" "style" "refactor" "test" "chore" "perf" "ci" "build" "revert")

# Initialize arrays
for type in "${TYPES[@]}"; do
    commits_by_type["$type"]=""
    authors_by_type["$type"]=""
done

# Process commits
process_commits() {
    local range="$1"
    
    # Git format: %H|%s|%an|%ad|%b
    git log "$range" --no-merges --pretty=format:"%H|%s|%an|%ad|%b" --date=short | while IFS='|' read -r hash subject author date body; do
        # Determine commit type
        local commit_type="other"
        for type in "${TYPES[@]}"; do
            if [[ "$subject" =~ ^$type(\([^)]+\))?: ]]; then
                commit_type="$type"
                break
            fi
        done
        
        # Check for breaking changes
        if is_breaking_change "$subject" "$body"; then
            local formatted=$(format_commit "$hash" "$subject" "$author" "$date")
            BREAKING_CHANGES+=("$formatted")
        fi
        
        # Format commit entry
        local formatted=$(format_commit "$hash" "$subject" "$author" "$date")
        
        # Add to appropriate type
        if [ -n "${commits_by_type[$commit_type]}" ]; then
            commits_by_type["$commit_type"]="${commits_by_type[$commit_type]}"$'\n'"$formatted"
        else
            commits_by_type["$commit_type"]="$formatted"
        fi
        
        # Track authors
        if [[ ! "${authors_by_type[$commit_type]}" =~ "$author" ]]; then
            if [ -n "${authors_by_type[$commit_type]}" ]; then
                authors_by_type["$commit_type"]="${authors_by_type[$commit_type]}, $author"
            else
                authors_by_type["$commit_type"]="$author"
            fi
        fi
    done
}

# Process commits based on range
if [ -n "$RANGE" ]; then
    process_commits "$RANGE"
else
    process_commits ""
fi

# Add breaking changes section if any
if [ ${#BREAKING_CHANGES[@]} -gt 0 ]; then
    echo "## ️ Breaking Changes" >> "$CHANGELOG_FILE"
    echo "" >> "$CHANGELOG_FILE"
    for change in "${BREAKING_CHANGES[@]}"; do
        echo "$change" >> "$CHANGELOG_FILE"
    done
    echo "" >> "$CHANGELOG_FILE"
fi

# Add sections for each commit type (in order of importance)
SECTION_ORDER=("feat" "fix" "perf" "refactor" "docs" "test" "ci" "build" "chore" "style" "revert" "other")

for type in "${SECTION_ORDER[@]}"; do
    if [ -n "${commits_by_type[$type]}" ] && [ "${commits_by_type[$type]}" != "" ]; then
        # Determine section title
        case "$type" in
            "feat") title=" Features" ;;
            "fix") title=" Bug Fixes" ;;
            "perf") title=" Performance Improvements" ;;
            "refactor") title="️ Refactoring" ;;
            "docs") title=" Documentation" ;;
            "test") title=" Tests" ;;
            "ci") title=" Continuous Integration" ;;
            "build") title="️ Build System" ;;
            "chore") title=" Chores" ;;
            "style") title=" Code Style" ;;
            "revert") title="↩️ Reverts" ;;
            "other") title=" Other Changes" ;;
            *) title="$type" ;;
        esac
        
        echo "## $title" >> "$CHANGELOG_FILE"
        
        # Add contributor count if available
        if [ -n "${authors_by_type[$type]}" ]; then
            local authors=(${authors_by_type[$type]//, / })
            local author_count=${#authors[@]}
            if [ $author_count -gt 0 ]; then
                echo "**Contributors:** ${authors_by_type[$type]}" >> "$CHANGELOG_FILE"
            fi
        fi
        
        echo "" >> "$CHANGELOG_FILE"
        echo "${commits_by_type[$type]}" >> "$CHANGELOG_FILE"
        echo "" >> "$CHANGELOG_FILE"
    fi
done

# Add footer with generation info
echo "---" >> "$CHANGELOG_FILE"
echo "" >> "$CHANGELOG_FILE"
echo "*Changelog generated automatically on $(date '+%Y-%m-%d %H:%M:%S')*" >> "$CHANGELOG_FILE"
if [ -n "$RANGE" ]; then
    echo "*Commit range: $RANGE*" >> "$CHANGELOG_FILE"
fi

echo -e "${GREEN} Changelog generated: $CHANGELOG_FILE${NC}"
echo ""
echo "Summary:"
echo "  - Total commits: $COMMIT_COUNT"
if [ ${#BREAKING_CHANGES[@]} -gt 0 ]; then
    echo "  - Breaking changes: ${#BREAKING_CHANGES[@]}"
fi

# Count commits by type
for type in "${TYPES[@]}"; do
    if [ -n "${commits_by_type[$type]}" ] && [ "${commits_by_type[$type]}" != "" ]; then
        local count=$(echo "${commits_by_type[$type]}" | grep -c '^-' || echo 0)
        if [ $count -gt 0 ]; then
            case "$type" in
                "feat") echo "  - Features: $count" ;;
                "fix") echo "  - Bug fixes: $count" ;;
                "docs") echo "  - Documentation: $count" ;;
                *) echo "  - $type: $count" ;;
            esac
        fi
    fi
done

echo ""
echo "Next steps:"
echo "  1. Review changelog: cat $CHANGELOG_FILE"
echo "  2. Include in release notes"
echo "  3. Commit if needed: git add $CHANGELOG_FILE"
