-- Add migration script here
INSERT INTO 
  realms (name, game_ip_address, game_port, rpc_ip_address, rpc_port)
VALUES (
  'CoEmu',
  '192.168.1.101',
  5816,
  '127.0.0.1',
  5817
) ON CONFLICT(name) DO NOTHING;
