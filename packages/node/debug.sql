-- Insert default admin user
INSERT
    OR IGNORE INTO [user] ([uuid], [username], [password])
VALUES
    (
        '00000000-0000-0000-0000-000000000001',
        'admin',
        '$argon2id$v=19$m=19456,t=2,p=1$Xes8LlbqVF5gTPFcpRUEWg$GFFfOvfIRWsLe2xDrvdnV1RB/876svMPntndRYrFe8s'
    );