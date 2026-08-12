ALTER TABLE snippets DROP COLUMN warning_type_id;
ALTER TABLE snippets DROP COLUMN warning_message_id;

DROP TABLE warning_types;
DROP TABLE warning_messages;
