### How to setup development sqlite db:

Install sqlx-cli (if not already):

```bash
cargo install sqlx-cli
```

Create sqlite database:

```bash
# set DATABASE_URL in `.env` if not already set
echo "DATABASE_URL=sqlite://$(pwd)/dev_db.sqlite" > .env

just create-dev-db
```
