CREATE TABLE items
(
    -- Item ID, for example `m2153223840`.
    id         TEXT PRIMARY KEY,

    -- Source listing JSON.
    listing    TEXT    NOT NULL,

    -- Update timestamp.
    updated_at INTEGER NOT NULL
);

CREATE TABLE subscriptions
(
    -- Base58-encoded UUID.
    id      TEXT PRIMARY KEY,

    chat_id INTEGER NOT NULL,

    query   TEXT    NOT NULL
);

CREATE TABLE notifications
(
    item_id    TEXT    NOT NULL REFERENCES items (id) ON UPDATE CASCADE ON DELETE CASCADE,
    chat_id    INTEGER NOT NULL REFERENCES subscriptions (id) ON UPDATE CASCADE ON DELETE CASCADE,
    message_id INTEGER NOT NULL,

    PRIMARY KEY (item_id, chat_id)
);
