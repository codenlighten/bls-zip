# Enterprise Multipass Database Migrations

This directory contains SQL migration scripts for the Enterprise Multipass system.

## Overview

The Enterprise Multipass system uses PostgreSQL as its database backend. These migrations create all necessary tables, indexes, constraints, and triggers for the seven core services:

1. Identity & Attestation Service
2. Wallet Service
3. Auth & SSO Service
4. Application Module Registry
5. Asset & Market Service
6. Event & Reporting Service
7. Hardware Pass Service

## Prerequisites

- PostgreSQL 14 or higher
- UUID extension (`uuid-ossp`)
- JSONB support (built into PostgreSQL)

## Migration Files

### `001_create_enterprise_tables.sql`
Creates all tables for the Enterprise Multipass system, including:

- **Identity Service**: `identity_profiles`, `kyc_verifications`, `attestations`
- **Wallet Service**: `wallet_accounts`, `wallet_balances`, `wallet_transactions`
- **Auth Service**: `multipass_credentials`, `multipass_sessions`
- **Application Service**: `application_modules`, `application_events`
- **Asset Service**: `asset_definitions`, `asset_balances`, `market_orders`, `positions`, `trades`
- **Event Service**: `notifications`, `report_definitions`, `generated_reports`
- **Hardware Service**: `hardware_passes`, `hardware_challenges`

### `002_rollback_enterprise_tables.sql`
Drops all tables created by the initial migration, useful for development and testing.

## Running Migrations

### Using psql (Manual)

```bash
# Apply migration
psql -U postgres -d enterprise_db -f migrations/001_create_enterprise_tables.sql

# Rollback (if needed)
psql -U postgres -d enterprise_db -f migrations/002_rollback_enterprise_tables.sql
```

### Using SQLx CLI (Recommended for Rust projects)

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Create database
sqlx database create --database-url "postgresql://postgres:password@localhost/enterprise_db"

# Run migrations
sqlx migrate run --database-url "postgresql://postgres:password@localhost/enterprise_db"

# Revert last migration
sqlx migrate revert --database-url "postgresql://postgres:password@localhost/enterprise_db"
```

### Using Environment Variables

Set the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgresql://postgres:password@localhost/enterprise_db"
sqlx migrate run
```

## Database Configuration

The Enterprise Multipass system expects the following environment variables:

- `DATABASE_URL`: Full PostgreSQL connection string
  - Example: `postgresql://user:password@localhost:5432/enterprise_db`
- `DATABASE_HOST`: Database host (default: `localhost`)
- `DATABASE_PORT`: Database port (default: `5432`)
- `DATABASE_NAME`: Database name (default: `enterprise_db`)
- `DATABASE_USER`: Database user
- `DATABASE_PASSWORD`: Database password
- `DATABASE_MAX_CONNECTIONS`: Connection pool size (default: `10`)

## Table Relationships

### Foreign Key Dependencies

```
identity_profiles (root)
├── kyc_verifications
├── attestations
├── wallet_accounts
│   ├── wallet_balances
│   ├── wallet_transactions
│   ├── asset_balances
│   ├── market_orders
│   └── positions
├── multipass_credentials
├── multipass_sessions
├── application_events
├── notifications
├── generated_reports
└── hardware_passes
    └── hardware_challenges

application_modules (root)
└── application_events

asset_definitions (root)
├── asset_balances
├── market_orders
├── positions
└── trades

report_definitions (root)
└── generated_reports
```

## Indexes

All tables include appropriate indexes for:
- Primary keys (automatic)
- Foreign keys
- Frequently queried columns (email, status, created_at, etc.)
- Unique constraints (username, email, symbol, etc.)

## Triggers

Automatic `updated_at` timestamp triggers are configured for tables that track modifications:
- `identity_profiles`
- `wallet_accounts`
- `multipass_credentials`
- `application_modules`
- `asset_definitions`
- `market_orders`
- `report_definitions`
- `hardware_passes`

## Data Types

### UUID
All primary keys use UUID v4 for globally unique identifiers.

### JSONB
Used for flexible metadata storage:
- `identity_profiles.metadata`
- `kyc_verifications.metadata`
- `attestations.metadata`
- `wallet_accounts.boundless_addresses`
- `wallet_transactions.metadata`
- `multipass_sessions.device_info`
- `asset_definitions.metadata`
- `notifications.metadata`
- `generated_reports.parameters`

### Timestamps
All timestamps use `TIMESTAMPTZ` (timestamp with timezone) for proper timezone handling.

### Arrays
PostgreSQL array types are used for:
- `wallet_accounts.labels` (TEXT[])
- `multipass_credentials.backup_codes` (TEXT[])
- `multipass_sessions.scopes` (TEXT[])
- `application_modules.required_scopes` (TEXT[])
- `report_definitions.parameters` (TEXT[])
- `hardware_passes.capabilities` (TEXT[])

## Security Considerations

1. **Password Hashing**: The `multipass_credentials.password_hash` column stores Argon2id hashes, never plain text passwords
2. **Token Security**: JWT tokens are hashed (SHA3-256) before storage in `multipass_sessions.token_hash`
3. **Chain Anchoring**: Attestations and reports can be anchored on the blockchain for immutability
4. **Soft Deletion**: Most tables use status fields or boolean flags rather than hard deletes to maintain audit trails

## Maintenance

### Vacuum and Analyze
Regular maintenance is recommended for optimal performance:

```sql
-- Vacuum all tables
VACUUM ANALYZE;

-- For specific tables with high write volume
VACUUM ANALYZE wallet_transactions;
VACUUM ANALYZE application_events;
VACUUM ANALYZE notifications;
```

### Monitoring
Monitor these high-traffic tables:
- `wallet_transactions` - Transaction history grows continuously
- `application_events` - Event tracking can grow rapidly
- `notifications` - User notifications accumulate
- `market_orders` - Trading activity
- `trades` - Trade history

Consider implementing data archival strategies for historical data.

## Development vs Production

### Development
- Use local PostgreSQL instance
- Enable query logging for debugging
- Use `002_rollback_enterprise_tables.sql` freely for testing

### Production
- Use managed PostgreSQL service (AWS RDS, Azure Database, etc.)
- Enable connection pooling (PgBouncer recommended)
- Configure automated backups
- Set up read replicas for reporting queries
- Enable point-in-time recovery (PITR)
- Monitor query performance and slow queries
- **Never** run rollback migrations on production data

## Troubleshooting

### Migration Fails
If a migration fails partway through:

1. Check the error message for the specific table/constraint
2. Fix the issue in the migration file
3. Run the rollback migration to clean up partial state
4. Re-run the forward migration

### Unique Constraint Violations
Ensure you're not trying to insert duplicate data for:
- `identity_profiles.email`
- `multipass_credentials.username`
- `asset_definitions.symbol`
- `application_modules.name`
- `report_definitions.name`

### Foreign Key Violations
Ensure parent records exist before creating child records:
- Create `identity_profiles` before `wallet_accounts`
- Create `asset_definitions` before `market_orders`
- Create `application_modules` before `application_events`

## Testing

To test migrations in a clean environment:

```bash
# Create test database
createdb enterprise_test

# Run migrations
psql -d enterprise_test -f migrations/001_create_enterprise_tables.sql

# Verify tables
psql -d enterprise_test -c "\dt"

# Rollback
psql -d enterprise_test -f migrations/002_rollback_enterprise_tables.sql

# Cleanup
dropdb enterprise_test
```

## Migration Checklist

Before applying migrations to production:

- [ ] Test migrations on a copy of production data
- [ ] Verify all foreign key relationships
- [ ] Check index coverage for common queries
- [ ] Estimate migration duration for large tables
- [ ] Plan for zero-downtime if required
- [ ] Backup database before migration
- [ ] Monitor application logs during migration
- [ ] Verify data integrity after migration

## Future Migrations

When adding new migrations:

1. Create files with sequential numbers: `003_add_feature.sql`, `004_modify_schema.sql`
2. Always include corresponding rollback migrations
3. Test both forward and rollback migrations
4. Document any breaking changes
5. Update this README with new table/column information

## Support

For issues related to migrations:
- Check the Enterprise Multipass documentation
- Review the table schemas in `001_create_enterprise_tables.sql`
- Verify environment variables are set correctly
- Check PostgreSQL logs for detailed error messages
