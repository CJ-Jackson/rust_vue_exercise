CREATE TABLE bucket_list
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name        TEXT                              NOT NULL,
    description TEXT                              NOT NULL,
    timestamp   TEXT                              NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO bucket_list (name, description)
VALUES ('Hello World', 'Test');