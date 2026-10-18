-- A decrypted original is kept in place of an encrypted upload (ADR 0013), so the file's own
-- hash (`file_sha256`) is no longer the hash of what the person uploaded. `upload_sha256`
-- keeps that, so uploading the same encrypted file again is still recognised.
ALTER TABLE finance.statements
    ADD COLUMN upload_sha256 text CHECK (upload_sha256 ~ '^[0-9a-f]{64}$'),
    -- The label (never the secret) of the Statement password that unlocked the file.
    ADD COLUMN unlocked_with_label text;
CREATE INDEX statements_upload_sha_idx ON finance.statements (upload_sha256);
