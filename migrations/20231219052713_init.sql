CREATE TABLE "subscriptions" (
    "title" TEXT PRIMARY KEY NOT NULL UNIQUE,
    "link" TEXT NOT NULL UNIQUE,
    "last" TEXT NOT NULL
);