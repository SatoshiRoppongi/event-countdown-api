-- 追加したフィールドとインデックスを削除
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_oauth;
ALTER TABLE users DROP COLUMN IF EXISTS password;
ALTER TABLE users DROP COLUMN IF EXISTS email;
ALTER TABLE users DROP COLUMN IF EXISTS oauth_id;
ALTER TABLE users DROP COLUMN IF EXISTS oauth_provider;