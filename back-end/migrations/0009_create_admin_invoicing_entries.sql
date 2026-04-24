CREATE TABLE admin_invoicing_entries (
    id          TEXT PRIMARY KEY,
    mongo_id    TEXT NOT NULL,
    date        DATE NOT NULL,
    price       INTEGER NOT NULL,
    note        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
