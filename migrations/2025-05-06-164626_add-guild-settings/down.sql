-- This file should undo anything in `up.sql`
ALTER TABLE servers
DROP COLUMN election_frequency;

ALTER TABLE servers
DROP COLUMN active;

ALTER TABLE servers
DROP COLUMN winner_temp_role_id;

ALTER TABLE servers
DROP COLUMN winner_perm_role_id;
