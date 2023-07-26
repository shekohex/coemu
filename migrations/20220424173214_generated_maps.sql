-- Add migration script here
INSERT INTO maps
VALUES (1000, 'desert.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'desert.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1001, 'd_antre01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'd_antre01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1002, 'newplain.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newplain.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1003, 'mine01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1004, 'forum.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'forum.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1005, 'arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1006, 'horse.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'horse.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1100, 'star01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1101, 'star02.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star02.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1102, 'star03.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star03.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1103, 'star04.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star04.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1104, 'star05.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star05.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1105, 'star10.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star10.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1106, 'star06.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star06.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1107, 'star07.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star07.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1108, 'star08.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star08.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1109, 'star09.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star09.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1007, 'smith.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'smith.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1008, 'grocery.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'grocery.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1009, 'grocery.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'grocery.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1010, 'newbie.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newbie.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1011, 'woods.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'woods.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1012, 'sky.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'sky.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1013, 'tiger.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'tiger.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1014, 'dragon.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'dragon.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1015, 'island.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'island.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1016, 'qiling.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'qiling.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1017, 'w-arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'w-arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1018, 'p-arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'p-arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1019, 'l-arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'l-arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1020, 'canyon.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'canyon.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1021, 'mine.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1022, 'brave.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'brave.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1025, 'mine-one.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-one.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1026, 'mine-two.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-two.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1027, 'mine-three.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-three.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1028, 'mine-four.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-four.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1029, 'mine-one2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-one2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1030, 'mine-two2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-two2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1031, 'mine-three2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-three2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1032, 'mine-four2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-four2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1035, 'newbie.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newbie.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (5000, 'mine-one.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-one.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (6000, 'prison.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'prison.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1036, 'street.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'street.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1037, 'faction-black.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction-black.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1038, 'faction.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1039, 'playground.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'playground.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1040, 'skycut.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'skycut.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1041, 'skymaze.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'skymaze.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (6001, 'prison.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'prison.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1042, 'lineup-pass.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup-pass.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1043, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1044, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1045, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1046, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1047, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1048, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1049, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1050, 'lineup.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'lineup.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1051, 'riskisland.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'riskisland.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1060, 'skymaze1.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'skymaze1.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1061, 'skymaze2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'skymaze2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1062, 'skymaze3.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'skymaze3.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1064, 'star.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1070, 'boa.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'boa.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1080, 'p-arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'p-arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1081, 'p-arena.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'p-arena.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1075, 'newcanyon.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newcanyon.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1076, 'newwoods.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newwoods.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1077, 'newdesert.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newdesert.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1078, 'newisland.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newisland.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1079, 'mys-island.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mys-island.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1063, 'riskisland.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'riskisland.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1082, 'idland-map.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'idland-map.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1090, 'parena-m.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'parena-m.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1091, 'parena-s.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'parena-s.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1098, 'house01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'house01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1099, 'house03.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'house03.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1601, 'sanctuary.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'sanctuary.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1201, 'task01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1202, 'task02.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task02.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1204, 'task04.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task04.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1205, 'task05.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task05.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1207, 'task07.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task07.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1208, 'task08.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task08.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1210, 'task10.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task10.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1211, 'task11.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'task11.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1212, 'island-snail.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'island-snail.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1213, 'desert-snail.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'desert-snail.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1214, 'canyon-fairy.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'canyon-fairy.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1215, 'woods-fairy.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'woods-fairy.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1216, 'newplain-fairy.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'newplain-fairy.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1500, 'mine-a.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-a.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1501, 'mine-b.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-b.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1502, 'mine-c.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-c.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1503, 'mine-d.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mine-d.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1351, 's-task01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 's-task01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1352, 's-task02.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 's-task02.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1353, 's-task03.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 's-task03.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1354, 's-task04.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 's-task04.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1505, 'slpk.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'slpk.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1506, 'hhpk.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'hhpk.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1507, 'blpk.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'blpk.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1508, 'ympk.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'ympk.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1509, 'mfpk.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'mfpk.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1550, 'faction01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1551, 'faction01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1510, 'grocery.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'grocery.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1511, 'forum.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'forum.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1512, 'tiger.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'tiger.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1615, 'jokul01.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'jokul01.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1616, 'tiemfiles.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'tiemfiles.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1617, 'tiemfiles.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'tiemfiles.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (2021, 'Dgate.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Dgate.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (2022, 'Dsquare.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Dsquare.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (2023, 'Dcloister.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Dcloister.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (2024, 'Dsigil.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Dsigil.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1645, 'cordiform.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'cordiform.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1560, 'faction.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1561, 'faction.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'faction.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1707, 'forum.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'forum.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1700, 'Gulf.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Gulf.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (601, 'Nhouse04.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Nhouse04.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (700, 'arena-none.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'arena-none.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1712, 'kunlun1.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun1.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1713, 'kunlun2.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun2.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1714, 'kunlun3.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun3.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1715, 'kunlun4.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun4.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1716, 'kunlun5.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun5.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1717, 'kunlun6.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun6.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1718, 'kunlun7.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun7.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1719, 'kunlun8.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun8.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1720, 'kunlun9.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'kunlun9.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1760, 'fairylandPK-07.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'fairylandPK-07.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1766, 'Halloween2007a.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Halloween2007a.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1767, 'Halloween2007boss.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Halloween2007boss.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1066, 'woods-z.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'woods-z.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1067, 'qiling-z.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'qiling-z.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1069, 'desert-snail.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'desert-snail.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1763, 'parena-m.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'parena-m.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1764, 'fairylandPK-03.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'fairylandPK-03.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1765, 'Nhouse04.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'Nhouse04.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1761, 'star05.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'star05.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
INSERT INTO maps
VALUES (1762, 'icecrypt-lev1.cmap', 0, 0, 0) ON CONFLICT (map_id) DO
UPDATE
SET path = 'icecrypt-lev1.cmap',
  revive_point_x = 0,
  revive_point_y = 0,
  flags = 0;
