CREATE TABLE IF NOT EXISTS maps(
    map_id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    revive_point_x INTEGER NOT NULL,
    revive_point_y INTEGER NOT NULL,
    flags INTEGER NOT NULL DEFAULT 0
);
