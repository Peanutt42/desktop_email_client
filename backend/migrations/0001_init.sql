PRAGMA foreign_keys = ON;

CREATE TABLE email_accounts (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	address TEXT NOT NULL,
	provider_type TEXT NOT NULL,

	CHECK (provider_type IN ('mock', 'manual_imap_smtp'))
);

CREATE TABLE email_folders (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	-- NULL <=> no parent
	parent_id INTEGER,
	email_account_id INTEGER NOT NULL,
    name TEXT NOT NULL,

    UNIQUE(parent_id, email_account_id, name),

    FOREIGN KEY (email_account_id) REFERENCES email_accounts (id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES email_folders (id) ON DELETE CASCADE
);
CREATE INDEX idx_email_folders_parent_id ON email_folders(parent_id);

CREATE TABLE emails (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	email_account_id INTEGER NOT NULL,
	author_name TEXT NOT NULL,
	author_address TEXT NOT NULL,
	subject TEXT NOT NULL,
	is_body_html BOOLEAN NOT NULL,
	body TEXT NOT NULL,
	-- NULL <=> no summary available
	body_summary VARCHAR(75) DEFAULT NULL,
	-- stored in UTC timezone
	sent_time DATETIME NOT NULL,
	read BOOLEAN NOT NULL DEFAULT FALSE,
	folder_id INTEGER NOT NULL,

	FOREIGN KEY (email_account_id) REFERENCES email_accounts (id) ON DELETE CASCADE,
	FOREIGN KEY (folder_id) REFERENCES email_folders (id) ON DELETE CASCADE
);
CREATE VIRTUAL TABLE emails_fts USING fts5(
    subject,
    author_name,
    author_address,
    body,
    content='emails',
    content_rowid='id',
    tokenize='trigram'
);
CREATE TRIGGER emails_fts_insert AFTER INSERT ON emails BEGIN
  INSERT INTO emails_fts(rowid, subject, author_name, author_address, body)
  VALUES (new.id, new.subject, new.author_name, new.author_address, new.body);
END;
CREATE TRIGGER emails_fts_delete AFTER DELETE ON emails BEGIN
  INSERT INTO emails_fts(emails_fts, rowid, subject, author_name, author_address, body)
  VALUES ('delete', old.id, old.subject, old.author_name, old.author_address, old.body);
END;

-- links emails with their tags
-- (tags are just folders, an email can only be in one folder, but it can have multiple folders as tags)
CREATE TABLE email_tags_refs (
    email_id INTEGER NOT NULL,
    -- tags and folders are the same
    folder_id INTEGER NOT NULL,

    PRIMARY KEY (email_id, folder_id),

    FOREIGN KEY (folder_id) REFERENCES email_folders (id) ON DELETE CASCADE
);
