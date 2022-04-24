CREATE TABLE IF NOT EXISTS realms (
    realm_id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL CHECK (length(name) <= 16),
    game_ip_address TEXT NOT NULL CHECK (length(game_ip_address) <= 16),
    game_port INTEGER NOT NULL CHECK (game_port >= 0 AND game_port <= 65535),
    rpc_ip_address INTEGER NOT NULL CHECK (length(rpc_ip_address) <= 16),
    rpc_port INTEGER NOT NULL CHECK (rpc_port >= 0 AND rpc_port <= 65535)
);
