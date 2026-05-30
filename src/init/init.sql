CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    organisation TEXT,
    location TEXT,
    salary TEXT,
    posted_date DATE,
    closing_date DATE,
    link TEXT NOT NULL,
    description TEXT,
    embedding FLOAT[],
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    telegram_id TEXT UNIQUE NOT NULL,
    cv_text TEXT,
    cv_embedding FLOAT8[],
    state TEXT,
    subscription_status TEXT DEFAULT 'free',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    subscription_expires_at TIMESTAMPTZ NULL,
    daily_requests INT DEFAULT 0,
    last_request_date DATE
);

CREATE TABLE IF NOT EXISTS user_sent_jobs (
    id SERIAL PRIMARY KEY,
    telegram_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(telegram_id, job_id)
);

