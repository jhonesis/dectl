# Ledger Pay

> *Core-profile fixture: the first level-1 heading declares the project name that the
> consumer derives group, package and artifact identifiers from.*
> *Tier: STANDARD*

## Functional Requirements

### REQ-001: Move money between own accounts
**User Story**:
> As an account holder, I want to move money between my own accounts, so that I can
> consolidate balances without leaving the application.

**Acceptance Criteria**:
- WHEN the source and target accounts belong to the same holder THEN the system SHALL accept the transfer
- WHEN the requested amount exceeds the source balance THEN the system SHALL reject the transfer with a declared error code
- WHEN the same idempotency key is submitted twice THEN the system SHALL return the first result without moving money again

**Implementation Notes**:
1. Declare `Account` and `Transfer` in `specs/data-model.md`
2. Declare `POST /transfers` in `specs/interface-contracts/cli.md`
