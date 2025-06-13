-- SNSログイン用のフィールドをusersテーブルに追加
ALTER TABLE users ADD COLUMN oauth_provider VARCHAR(50);
ALTER TABLE users ADD COLUMN oauth_id VARCHAR(255);

-- 独自ログイン用にemailとpasswordフィールドを追加
ALTER TABLE users ADD COLUMN email VARCHAR(255);
ALTER TABLE users ADD COLUMN password VARCHAR(255);

-- OAuth用のユニークインデックスを追加
CREATE UNIQUE INDEX idx_users_oauth ON users(oauth_provider, oauth_id) WHERE oauth_provider IS NOT NULL AND oauth_id IS NOT NULL;

-- email用のユニークインデックスを追加
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE email IS NOT NULL;