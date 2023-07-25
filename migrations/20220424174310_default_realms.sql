-- Add migration script here
INSERT INTO 
  realms (name, game_ip_address, game_port)
VALUES (
  'CoEmu',
  '192.168.1.101',
  5816
) ON CONFLICT(name) DO NOTHING;
