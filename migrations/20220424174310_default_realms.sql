-- Add migration script here
INSERT INTO realms (name, game_ip_address, game_port)
VALUES (
    'CoEmu',
    '192.168.0.110',
    -- Change this to your server's IP address
    5816
  ) ON CONFLICT(name) DO NOTHING;
