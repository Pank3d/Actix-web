CREATE TABLE IF NOT EXISTS "last_signatures" (
	"pubkey" BYTEA,
	"signature" BYTEA,
);

CREATE TABLE IF NOT EXISTS "messages" (
	"sender_x" BYTEA,
	"sender_p" SMALLINT,
	"receiver_x" BYTEA,
	"receiver_p" SMALLINT,
	"timestamp" TIMESTAMP,
	"data" BYTEA,
);