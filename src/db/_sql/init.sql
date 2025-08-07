PRAGMA foreign_keys = ON;

CREATE TABLE users
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT                              NOT NULL,
    password BLOB                              NOT NULL
);

CREATE TABLE user_login_tokens
(
    user_id      INTEGER     NOT NULL,
    token        TEXT UNIQUE NOT NULL,
    expire_after TEXT        NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE bucket_list
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name        TEXT                              NOT NULL,
    description TEXT                              NOT NULL,
    timestamp   TEXT                              NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO bucket_list (name, description)
VALUES ('Hello World', 'Test');