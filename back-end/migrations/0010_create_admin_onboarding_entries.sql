CREATE TYPE currencies AS ENUM ('czk', 'eur');

CREATE TABLE admin_onboarding_entries (
    id              TEXT PRIMARY KEY,
    mongo_id        TEXT NOT NULL,
    date_training   DATE,
    paid            BOOLEAN NOT NULL DEFAULT FALSE,
    price           INTEGER NOT NULL,
    currency        currencies NOT NULL,
    invoiced        BOOLEAN NOT NULL DEFAULT FALSE,
    invoiced_date   DATE,
    business_module BOOLEAN NOT NULL DEFAULT FALSE,
    fans_module     BOOLEAN NOT NULL DEFAULT FALSE,
    note            TEXT,
    enigoo_involved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
