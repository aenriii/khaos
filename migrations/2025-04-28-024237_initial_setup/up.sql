-- servers is created first as it has no dependencies
CREATE TABLE servers (
    id TEXT PRIMARY KEY NOT NULL,
    -- these should both end up very easily set
    announcements_channel_id TEXT, -- id of the channel where announcements will be posted
    poll_channel_id TEXT -- id of the channel where polls will be posted
);

CREATE TABLE elections (
    -- i love uuidv7!!!
    uuid TEXT PRIMARY KEY NOT NULL,
    -- id of the server where the election is taking place
    server_id TEXT NOT NULL REFERENCES servers (id) ON DELETE CASCADE,
    -- id of the message holding the poll
    poll_message_id TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (
        status IN ('scheduled', 'active', 'finished', 'cancelled')
    )
);

CREATE TABLE nominees (
    id SERIAL PRIMARY KEY NOT NULL,
    election_id TEXT NOT NULL REFERENCES elections (uuid) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    poll_option_text TEXT NOT NULL,
    votes_received INTEGER NULL,
    nomination_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        nomination_status IN ('pending', 'accepted', 'declined')
    ),
    CONSTRAINT unique_nomination UNIQUE (election_id, user_id)
);
