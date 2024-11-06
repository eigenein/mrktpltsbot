CREATE TABLE search_queries
(
    -- SeaHash'ed search QUERY,
    hash INTEGER PRIMARY KEY,

    text TEXT NOT NULL
);

-- Marktplaats items.
CREATE TABLE items
(
    -- Item ID, for example `m2153223840`.
    id         TEXT PRIMARY KEY,

    -- Update timestamp.
    updated_at INTEGER NOT NULL
);

-- Search QUERY subscriptions.
CREATE TABLE subscriptions
(
    chat_id    INTEGER NOT NULL,
    query_hash INTEGER NOT NULL REFERENCES search_queries (hash) ON UPDATE CASCADE ON DELETE CASCADE,

    PRIMARY KEY (chat_id, query_hash)
);

-- Sent notifications.
CREATE TABLE notifications
(
    item_id TEXT    NOT NULL REFERENCES items (id) ON UPDATE CASCADE ON DELETE CASCADE,
    chat_id INTEGER NOT NULL,

    PRIMARY KEY (item_id, chat_id)
);
