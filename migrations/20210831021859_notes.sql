CREATE TABLE IF NOT EXISTS "notes" (
	"chat_id" BIGINT,
	"note_id" TEXT,
	"note_content" TEXT NOT NULL,
	PRIMARY KEY("chat_id", "note_id"),
	CONSTRAINT "fk_notes" FOREIGN KEY ("chat_id") REFERENCES "chats" ("chat_id")
);