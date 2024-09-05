CREATE TABLE items
(
    -- Item ID, for example `m2153223840`.
    id         TEXT PRIMARY KEY,

    -- Source listing JSON.
    listing    TEXT    NOT NULL,

    -- Update timestamp.
    updated_at INTEGER NOT NULL
);
