CREATE TABLE IF NOT EXISTS maps(
    map_id INT PRIMARY KEY UNIQUE,
    path TEXT NOT NULL,
    revive_point_x INT NOT NULL,
    revive_point_y INT NOT NULL,
    flags INT NOT NULL DEFAULT 0
)
