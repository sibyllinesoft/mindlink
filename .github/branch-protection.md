# Branch Protection Configuration

This document describes the recommended branch protection rules for the MindLink repository to enforce the CI/CD quality gates and maintain code quality.

## Required Branch Protection Rules

### Main Branch (`main`)

**Required Status Checks:**
- `Quality Gates & Validation / frontend-quality`
- `Quality Gates & Validation / backend-quality` 
- `Quality Gates & Validation / testing`
- `Quality Gates & Validation / coverage`
- `Quality Gates & Validation / security`
- `Build Validation / build-validation (ubuntu-22.04)`
- `Build Validation / build-validation (windows-latest)`
- `Build Validation / build-validation (macos-latest)`
- `Build Validation / bundle-analysis`

**Additional Protection Settings:**
- ✅ **Require branches to be up to date before merging**
- ✅ **Require status checks to pass before merging**
- ✅ **Restrict pushes that create files larger than 100MB**
- ✅ **Require signed commits**
- ✅ **Require pull request reviews before merging** (minimum 1 review)
- ✅ **Dismiss stale PR approvals when new commits are pushed**
- ✅ **Require review from CODEOWNERS**
- ❌ **Allow force pushes**
- ❌ **Allow deletions**

### Develop Branch (`develop`)

**Required Status Checks:**
- Same as main branch (all quality gates and build validation)

**Additional Protection Settings:**
- ✅ **Require branches to be up to date before merging**
- ✅ **Require status checks to pass before merging**
- ✅ **Require pull request reviews before merging** (minimum 1 review)
- ✅ **Restrict pushes that create files larger than 100MB**
- ❌ **Allow force pushes**
- ❌ **Allow deletions**

## GitHub CLI Configuration Script

Run this script to automatically configure branch protection rules:

```bash
#!/bin/bash
# configure-branch-protection.sh

set -e

REPO="mindlink/mindlink"  # Update with your repository
BRANCH_MAIN="main"
BRANCH_DEVELOP="develop"

echo "🔧 Configuring branch protection for $REPO..."

# Main branch protection
echo "Setting up protection for $BRANCH_MAIN..."
gh api repos/$REPO/branches/$BRANCH_MAIN/protection \
  --method PUT \
  --field required_status_checks='{
    "strict": true,
    "contexts": [
      "Quality Gates & Validation / frontend-quality",
      "Quality Gates & Validation / backend-quality",
      "Quality Gates & Validation / testing", 
      "Quality Gates & Validation / coverage",
      "Quality Gates & Validation / security",
      "Build Validation / build-validation (ubuntu-22.04)",
      "Build Validation / build-validation (windows-latest)", 
      "Build Validation / build-validation (macos-latest)",
      "Build Validation / bundle-analysis"
    ]
  }' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "require_last_push_approval": false
  }' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false \
  --field block_creations=false

# Develop branch protection
echo "Setting up protection for $BRANCH_DEVELOP..."
gh api repos/$REPO/branches/$BRANCH_DEVELOP/protection \
  --method PUT \
  --field required_status_checks='{
    "strict": true,
    "contexts": [
      "Quality Gates & Validation / frontend-quality",
      "Quality Gates & Validation / backend-quality", 
      "Quality Gates & Validation / testing",
      "Quality Gates & Validation / coverage",
      "Quality Gates & Validation / security",
      "Build Validation / build-validation (ubuntu-22.04)",
      "Build Validation / build-validation (windows-latest)",
      "Build Validation / build-validation (macos-latest)", 
      "Build Validation / bundle-analysis"
    ]
  }' \
  --field enforce_admins=false \
  --field required_pull_request_reviews='{
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "require_last_push_approval": false
  }' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false \
  --field block_creations=false

echo "✅ Branch protection configured successfully!"
echo ""
echo "📋 Summary:"
echo "- Main branch: Strict protection with code owner review required"
echo "- Develop branch: Standard protection with team review required"
echo "- All quality gates enforced on both branches"
echo "- Force pushes and deletions blocked"
echo ""
echo "🔗 Verify settings at: https://github.com/$REPO/settings/branches"
```

## Manual Configuration Steps

If you prefer to configure via GitHub UI:

### 1. Navigate to Repository Settings
Go to `https://github.com/[username]/mindlink/settings/branches`

### 2. Add Branch Protection Rule for `main`
1. Click "Add rule"
2. Branch name pattern: `main`
3. Enable "Require status checks to pass before merging"
4. Enable "Require branches to be up to date before merging"
5. Add all required status checks listed above
6. Enable "Require pull request reviews before merging"
   - Required approving reviews: 1
   - Dismiss stale reviews: ✅
   - Require review from CODEOWNERS: ✅
7. Enable "Restrict pushes that create files larger than 100MB"
8. Disable "Allow force pushes" and "Allow deletions"

### 3. Add Branch Protection Rule for `develop`
Repeat the same process with slightly relaxed rules (no CODEOWNER requirement).

## CODEOWNERS Configuration

Create `.github/CODEOWNERS` file:

```
# Global owners for all files
* @maintainer-username

# Rust backend code
/src-tauri/ @backend-team-or-maintainer

# Frontend code
/src/ @frontend-team-or-maintainer

# CI/CD and infrastructure
/.github/ @devops-team-or-maintainer
/scripts/ @devops-team-or-maintainer

# Documentation
/docs/ @documentation-team-or-maintainer
*.md @documentation-team-or-maintainer
```

## Enforcement Benefits

With these protection rules in place:

✅ **Quality Assurance**: No code reaches main without passing all quality gates
✅ **Security**: All commits must pass security audits and vulnerability scans
✅ **Cross-platform**: Builds must work on all supported platforms
✅ **Code Review**: All changes reviewed by appropriate team members
✅ **Consistency**: Formatting, linting, and testing standards enforced
✅ **Documentation**: Changes must not break documentation builds
✅ **Performance**: Bundle size and performance regression detection

## Troubleshooting

### Common Issues

**Status Check Not Found:**
- Verify the exact status check name in GitHub Actions
- Status checks must complete successfully at least once before protection can reference them

**Cannot Merge Despite Passing Checks:**
- Ensure branch is up to date with target branch
- Verify all required reviews are in place

**Admin Bypass Not Working:**
- Admin enforcement is enabled on main branch
- Even administrators must follow the rules

### Testing Protection Rules

1. Create a test branch with failing code
2. Open a PR to main/develop
3. Verify that merge is blocked until quality gates pass
4. Verify that required reviews prevent merge

## Monitoring and Metrics

Track these metrics to ensure protection is effective:

- **PR Merge Success Rate**: Target >95% after quality gates pass
- **Time to Merge**: Should be predictable with automated gates
- **Quality Gate Failure Rate**: Track common failure types to improve
- **Security Issue Prevention**: Zero critical security issues in main
- **Build Failure Rate**: <5% in main branch (protected by validation)

This comprehensive protection ensures enterprise-grade quality while maintaining developer productivity through fast, automated feedback loops.