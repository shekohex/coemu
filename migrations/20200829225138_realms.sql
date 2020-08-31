-- Add migration script here
CREATE TABLE IF NOT EXISTS realms (
    realm_id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(16) UNIQUE NOT NULL,
    game_ip_address INET NOT NULL,
    game_port SMALLINT NOT NULL,
    rpc_ip_address INET NOT NULL,
    rpc_port SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
)
