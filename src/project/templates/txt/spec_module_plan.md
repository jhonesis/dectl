# Implementation Plan: [MODULE_NAME]
> *Technology-specific. Describes HOW to build this module.*
> *Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## References
- Implements: `specs/[MODULE_NAME]/spec.md`
- Constitution: `specs/[MODULE_NAME]/constitution.md`

## Tech Stack
(inherits from root project)

## Architecture
(module internal architecture)

## Implementation Phases

### Phase 1: Foundation

**Build Gate**: `[build command]` — must pass with 0 errors
**Verify Gate**: `[verify command]`
**Rule**: Each task must compile and verify before the next begins

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
