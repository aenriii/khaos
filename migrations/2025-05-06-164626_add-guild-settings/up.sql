-- election_frequency is in hours,
-- default is once every 2 weeks
ALTER TABLE servers ADD election_frequency INTEGER NOT NULL DEFAULT 336;

-- whether or not a server has automatic
-- elections active, defaults to false.
ALTER TABLE servers ADD active BOOLEAN NOT NULL DEFAULT false;

-- role id for the current election winner
ALTER TABLE servers ADD winner_temp_role_id TEXT NULL;

-- role id for all election winners,
-- a kind of "you won sometime !" reward
ALTER TABLE servers ADD winner_perm_role_id TEXT NULL;
