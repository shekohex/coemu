CREATE TABLE IF NOT EXISTS portals (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    from_map_id INT NOT NULL CONSTRAINT fk_map_from REFERENCES maps(map_id) ON DELETE CASCADE,
    from_x SMALLINT NOT NULL,
    from_y SMALLINT NOT NULL,
    to_map_id INT NOT NULL CONSTRAINT fk_map_to REFERENCES maps(map_id) ON DELETE CASCADE,
    to_x SMALLINT NOT NULL,
    to_y SMALLINT NOT NULL
)
