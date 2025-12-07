#!/bin/bash
# Script: check-changelog.sh
# Purpose: Verify that CHANGELOG.md is updated when code files are modified in a PR
# Usage: ./check-changelog.sh
#
# Exit codes:
#   0 = All checks passed
#   1 = Changelog check failed
#   2 = Configuration error
#
# Environment variables:
#   GITHUB_BASE_REF: Base branch (set by GitHub Actions)
#   GITHUB_HEAD_REF: Head branch (set by GitHub Actions)
#   GITHUB_EVENT_PATH: Event payload path (set by GitHub Actions)

set -o errexit
set -o pipefail

# ============================================================================
# Configuration: File Patterns
# ============================================================================

# Code files that require CHANGELOG.md update
CODE_PATTERNS=(
  "src/**"
  "crates/**"
  "examples/**"
  "benches/**"
  "tests/**"
  "Cargo.toml"
  "Makefile"
)

# Files/patterns exempt from CHANGELOG requirement (pure docs/config)
EXEMPT_PATTERNS=(
  "docs/**"
  ".github/workflows/**"
  ".gitignore"
  ".nvmrc"
  ".prettierrc*"
  ".eslintrc*"
  "*.md"  # Root-level markdown files like README.md, CONTRIBUTING.md
  "LICENSE"
  "CODE_OF_CONDUCT.md"
  ".github/pull_request_template.md"
)

CHANGELOG_FILE="CHANGELOG.md"

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
  echo "ℹ️  $*"
}

log_success() {
  echo "✅ $*"
}

log_error() {
  echo "❌ $*"
}

log_warning() {
  echo "⚠️  $*"
}

# Check if a file matches any pattern in an array
# Args: $1 = filename, $2 = array name
matches_pattern() {
  local file="$1"
  local pattern_array_name="$2"
  
  # Use indirect reference to get array
  local patterns_ref="${pattern_array_name}[@]"
  local patterns=("${!patterns_ref}")
  
  for pattern in "${patterns[@]}"; do
    # Convert glob pattern to regex for matching
    # Simple conversion: ** → .*, * → [^/]*
    local regex="${pattern//\*\*/.*}"
    regex="${regex//\*/[^/]*}"
    
    if [[ "$file" =~ ^${regex}$ ]]; then
      return 0
    fi
  done
  
  return 1
}

# Check if a file is code-related (not exempt)
is_code_file() {
  local file="$1"
  
  # First check if it matches exempt patterns
  if matches_pattern "$file" "EXEMPT_PATTERNS"; then
    return 1  # 1 = false (exempt, not code)
  fi
  
  # Then check if it matches code patterns
  if matches_pattern "$file" "CODE_PATTERNS"; then
    return 0  # 0 = true (is code)
  fi
  
  # Default: if it doesn't match code patterns, it's not code
  return 1
}

# Print exemption rules for help message
print_exemption_rules() {
  cat << 'EOF'

📋 File Pattern Rules
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ These changes REQUIRE CHANGELOG.md update:
   • src/** (source code)
   • crates/** (Rust crate changes)
   • examples/** (example code)
   • benches/** (benchmark code)
   • tests/** (test code)
   • Cargo.toml (dependency updates)
   • Makefile (build system)

⏭️  These changes are EXEMPT from CHANGELOG requirement:
   • docs/** (pure documentation)
   • .github/workflows/** (CI configuration)
   • *.md files at root (README, CONTRIBUTING, LICENSE, etc.)
   • .gitignore, .nvmrc (infrastructure config)

📖 For guidance on changelog entries, see:
   👉 CONTRIBUTING.md#maintaining-changelogmd
   👉 ADR 0010: Maintain CHANGELOG.md

📝 CHANGELOG.md Entry Format:
   - YYYY-MM-DD - Author Name - [category] description
   
   Example:
   - 2025-12-07 - Jane Doe - [feature] Added CI guard for CHANGELOG.md
   - 2025-12-07 - auto-llm:copilot - [fix] Fixed edge case in routing

EOF
}

# ============================================================================
# Main Logic
# ============================================================================

main() {
  log_info "Checking CHANGELOG.md requirement for PR changes..."
  
  # Verify GitHub Actions environment - only base_ref required
  if [[ -z "$GITHUB_BASE_REF" ]]; then
    log_error "Missing GITHUB_BASE_REF environment variable"
    log_error "This script should only run in pull_request GitHub Actions events"
    return 2
  fi
  
  # Check if skip label is present
  if [[ -n "$GITHUB_EVENT_PATH" ]] && [[ -f "$GITHUB_EVENT_PATH" ]]; then
    # Extract PR labels from GitHub event payload
    SKIP_LABEL_PRESENT=$(jq -r '.pull_request.labels[] | select(.name == "skip-changelog-check")' "$GITHUB_EVENT_PATH" 2>/dev/null || echo "")
    
    if [[ -n "$SKIP_LABEL_PRESENT" ]]; then
      log_warning "CHANGELOG.md check SKIPPED due to 'skip-changelog-check' label"
      log_info "This label should only be used for emergency fixes"
      return 0
    fi
  fi
  
  # Get list of files changed in PR
  # Use merge base to compare against base branch (handles both direct and fork PRs)
  log_info "Computing changed files against base branch: $GITHUB_BASE_REF"
  
  # Find merge base and diff against it
  MERGE_BASE=$(git merge-base "origin/$GITHUB_BASE_REF" HEAD 2>/dev/null)
  if [[ -z "$MERGE_BASE" ]]; then
    log_error "Could not find merge base between origin/$GITHUB_BASE_REF and HEAD"
    log_info "Available refs:"
    git show-ref | head -20
    return 2
  fi
  
  log_info "Merge base: $MERGE_BASE"
  CHANGED_FILES=$(git diff --name-only "$MERGE_BASE...HEAD" 2>/dev/null || echo "")
  
  if [[ -z "$CHANGED_FILES" ]]; then
    log_error "No files detected in diff. This should not happen in a normal PR."
    log_info "Git output for debugging:"
    git diff --name-only "$MERGE_BASE...HEAD" || true
    git log --oneline "$MERGE_BASE..HEAD" || true
    return 2
  fi
  
  # Separate changed files into code and exempt categories
  CODE_FILES=()
  EXEMPT_FILES=()
  CHANGELOG_MODIFIED=false
  
  while IFS= read -r file; do
    if [[ "$file" == "$CHANGELOG_FILE" ]]; then
      CHANGELOG_MODIFIED=true
    elif is_code_file "$file"; then
      CODE_FILES+=("$file")
    else
      EXEMPT_FILES+=("$file")
    fi
  done <<< "$CHANGED_FILES"
  
  # Display summary
  log_info "Changed files summary:"
  log_info "  Code files requiring CHANGELOG: ${#CODE_FILES[@]}"
  log_info "  Exempt files: ${#EXEMPT_FILES[@]}"
  log_info "  CHANGELOG.md modified: $CHANGELOG_MODIFIED"
  
  # Show details if verbose
  if [[ "${VERBOSE:-}" == "true" ]]; then
    if (( ${#CODE_FILES[@]} > 0 )); then
      log_info "Code files changed:"
      printf '    - %s\n' "${CODE_FILES[@]}"
    fi
    if (( ${#EXEMPT_FILES[@]} > 0 )); then
      log_info "Exempt files changed:"
      printf '    - %s\n' "${EXEMPT_FILES[@]}"
    fi
  fi
  
  # Determine if CHANGELOG.md is required
  if (( ${#CODE_FILES[@]} == 0 )); then
    # Only exempt files were changed
    log_success "No code changes detected. CHANGELOG.md not required."
    log_info "All changed files are documentation, config, or infrastructure."
    return 0
  fi
  
  # Code files were changed
  if [[ "$CHANGELOG_MODIFIED" != "true" ]]; then
    log_error "CHANGELOG.md must be updated for code changes!"
    log_error ""
    log_error "Changed code files:"
    printf '  • %s\n' "${CODE_FILES[@]}"
    print_exemption_rules
    return 1
  fi
  
  # Both code changed and CHANGELOG.md was updated
  log_success "Code changes detected and CHANGELOG.md was updated."
  log_info "${#CODE_FILES[@]} code file(s) changed in this PR."
  return 0
}

# Run main function
main "$@"
exit $?
