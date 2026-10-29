# Data Model

> *Core-profile fixture: entity tables use exactly the columns the consumer parses:
> `Field / Type / Required / Indexes / Description`.*

## Entities

### Account
| Field | Type | Required | Indexes | Description |
|---|---|---|---|---|
| id | uuid | ✅ | PK | Primary key |
| holder_id | uuid | ✅ | INDEX | Owner of the account |
| balance | decimal | ✅ | | Available balance |
| currency | string | ✅ | | ISO-4217 currency code |
| created_at | timestamp | ✅ | | Creation instant (UTC) |

### Transfer
| Field | Type | Required | Indexes | Description |
|---|---|---|---|---|
| id | uuid | ✅ | PK | Primary key |
| source_account_id | uuid | ✅ | INDEX | FK → Account.id |
| target_account_id | uuid | ✅ | INDEX | FK → Account.id |
| amount | decimal | ✅ | | Transferred amount |
| idempotency_key | string | ✅ | UNIQUE | Client-supplied replay key |
| status | string | ✅ | | PENDING, SETTLED or REJECTED |
| created_at | timestamp | ✅ | | Creation instant (UTC) |

## Relationships
- Account has many Transfer (via foreign keys `source_account_id` and `target_account_id`)
- Transfer belongs to Account

## Migration Notes
Greenfield fixture: no migration required.
