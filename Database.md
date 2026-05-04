 sudo -u postgres psql
CREATE DATABASE job_agent;
CREATE USER job_user WITH PASSWORD 'strongpassword';
GRANT ALL PRIVILEGES ON DATABASE job_agent TO job_user;
\l -> returns created database;
\du -> returns user

Connecting to job agnt
\c job_agent

Creating table

CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    organisation TEXT,
    location TEXT,
    salary TEXT,
    posted_date DATE,
    closing_date DATE,
    link TEXT NOT NULL,
    description TEXT,

    last_seen_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

ALTER TABLE jobs ADD COLUMN embedding FLOAT[];

DELETE FROM jobs - this cleans the database

connection string 

DATABASE_URL=postgres://job_user:strongpassword@localhost/job_agent

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    telegram_id TEXT UNIQUE NOT NULL,
    cv_text TEXT,
    cv_embedding FLOAT8[],
    created_at TIMESTAMP DEFAULT NOW()
);
SELECT * FROM users LIMIT 10; 

ALTER TABLE users 
ADD COLUMN subscription_status TEXT DEFAULT 'free',
ADD COLUMN subscription_expires_at TIMESTAMP NULL,
ADD COLUMN daily_requests INT DEFAULT 0,
ADD COLUMN last_request_date DATE;

CREATE TABLE user_sent_jobs (
    id SERIAL PRIMARY KEY,
    telegram_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(telegram_id, job_id)
);

\dt

ALTER SEQUENCE user_sent_jobs_id_seq OWNER TO job_user;

ALTER TABLE user_sent_jobs OWNER TO job_user;