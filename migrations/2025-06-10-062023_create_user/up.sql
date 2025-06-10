-- user テーブル
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    social_id VARCHAR(255),
    avatar_url TEXT,
    region VARCHAR(100),
    gender VARCHAR(50),
    profile TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
