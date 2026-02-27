-- Create user table
CREATE TABLE [user] (
    [uuid] TEXT NOT NULL,
    [username] TEXT NOT NULL UNIQUE,
    [password] TEXT NOT NULL,
    PRIMARY KEY ([uuid])
);

-- Create session table
CREATE TABLE [session] (
    [token] TEXT NOT NULL,
    [user] TEXT NOT NULL,
    [user_agent] TEXT NOT NULL,
    PRIMARY KEY ([token]),
    FOREIGN KEY ([user]) REFERENCES [user]([uuid]) ON DELETE CASCADE
)