-- Create user table
CREATE TABLE [user] (
    [uuid] TEXT NOT NULL,
    [username] TEXT NOT NULL UNIQUE,
    -- Password will be null for users that have been invited but haven't set a password yet
    [password] TEXT NULL,
    [join_token] TEXT NULL UNIQUE,
    PRIMARY KEY ([uuid])
);

-- Create session table
CREATE TABLE [session] (
    [uuid] TEXT NOT NULL,
    [token] TEXT NOT NULL UNIQUE,
    [user] TEXT NOT NULL,
    [user_agent] TEXT NOT NULL,
    PRIMARY KEY ([uuid]),
    FOREIGN KEY ([user]) REFERENCES [user]([uuid]) ON DELETE CASCADE
)