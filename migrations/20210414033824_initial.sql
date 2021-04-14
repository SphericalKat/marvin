CREATE TABLE IF NOT EXISTS "users" (
    "user_id" BIGINT PRIMARY KEY,
    "user_name" TEXT UNIQUE,
    "full_name" TEXT NOT NULL
);

CREATE INDEX "idx_users_user_name" ON "users" ("user_name");

CREATE TABLE IF NOT EXISTS "chats" (
    "chat_id" BIGINT PRIMARY KEY,
    "chat_name" TEXT
);
