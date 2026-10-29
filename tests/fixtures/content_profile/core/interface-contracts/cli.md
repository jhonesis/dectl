# Interface Contracts

## Interface Type

- **`type: cli`** — endpoints are declared as `### <METHOD> /<path>` headings, one per operation.

---

## Endpoints

### GET /accounts
**Description**: List the accounts owned by the caller
**Requirements**: REQ-001

**Response 200**:
```json
[
  {
    "id": "uuid",
    "balance": "0.00",
    "currency": "EUR"
  }
]
```

### GET /accounts/{id}
**Description**: Read one account owned by the caller
**Requirements**: REQ-001

### POST /transfers
**Description**: Move money between two accounts owned by the caller
**Requirements**: REQ-001
**Idempotency**: Requires an `Idempotency-Key`; a replay returns the original result.

**Request Body**:
```json
{
  "source_account_id": "uuid",
  "target_account_id": "uuid",
  "amount": "10.00",
  "idempotency_key": "string"
}
```

**Response 201**:
```json
{
  "id": "uuid",
  "status": "PENDING"
}
```
