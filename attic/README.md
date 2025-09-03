# Attic Directory

This directory contains quarantined code that has been identified for potential removal during repository cleanup. Files here are preserved temporarily to allow for restoration if needed.

## Contents

- Files moved here during cleanup process
- Reason for quarantine is documented in commit messages
- Files will be permanently deleted after one release cycle if not restored

## Restoration

To restore a file from attic:
```bash
git mv attic/filename original/location/filename
git commit -m "restore: bring back filename from attic"
```

## Cleanup Schedule

Files in attic are reviewed for permanent deletion during major releases.