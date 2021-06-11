CREATE TABLE alert
(
    alert_id      TEXT   NOT NULL,
    url           TEXT   NOT NULL,
    matching_text TEXT   NOT NULL,
    discord_id    BIGINT NOT NULL,

    PRIMARY KEY (alert_id)
);
