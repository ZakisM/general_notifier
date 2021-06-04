CREATE TABLE user
(
    hashed_token TEXT   NOT NULL,
    discord_id   BIGINT NOT NULL,
    discord_name TEXT   NOT NULL,

    PRIMARY KEY (hashed_token)
);
