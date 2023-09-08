CREATE TABLE "user" (
    user_id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE
);

INSERT INTO "user" (user_id, username, email)
SELECT
    md5(random()::text || clock_timestamp()::text)::uuid,
    'user' || generate_series,
    'user' || generate_series || '@example.com'
FROM generate_series(1, 2000);
