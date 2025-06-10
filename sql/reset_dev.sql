-- テーブルのデータ削除（依存関係に注意）
TRUNCATE TABLE
  reports,
  favorites,
  comments,
  event_tags,
  events,
  tags,
  users
RESTART IDENTITY CASCADE;

-- 初期データの挿入
-- users
INSERT INTO users (name, social_id, avatar_url, region, gender, profile)
VALUES 
  ('ゆう', 'google|1234567890', 'https://example.com/avatar1.jpg', '埼玉県', 'その他', 'イベントが大好きです！'),
  ('あかり', 'twitter|abcdef123456', 'https://example.com/avatar2.jpg', '東京都', '女性', 'アイドル推しです！'),
  ('たけし', NULL, NULL, '大阪府', '男性', 'スポーツ観戦が趣味');

-- tags
INSERT INTO tags (name)
VALUES 
  ('花火'), ('アイドル'), ('スポーツ'), ('音楽'), ('グルメ');

-- events
INSERT INTO events (event_type, name, start_date, end_date, description, location, source_type, url, image_url)
VALUES
  ('花火', '隅田川花火大会2025', '2025-07-28', '2025-07-28', '東京の夏の風物詩、隅田川花火大会。', '東京都墨田区', 'api', 'https://hanabi.example.com/sumidagawa', 'https://images.example.com/sumidagawa.jpg'),
  ('アイドル', 'アイドルフェス2025', '2025-08-15', '2025-08-16', '人気アイドルが集結する夏フェス！', '幕張メッセ', 'custom', 'https://idolfes.example.com', 'https://images.example.com/idolfes.jpg');

-- event_tags
INSERT INTO event_tags (event_id, tag_id)
VALUES
  (1, 1), (1, 5), (2, 2), (2, 4);

-- comments
INSERT INTO comments (user_id, event_id, content)
VALUES 
  (1, 1, '去年行ってとても楽しかったです！今年も楽しみ！'),
  (2, 2, '出演者のラインナップが最高！');

-- favorites
INSERT INTO favorites (user_id, event_id)
VALUES 
  (1, 1), (2, 2), (3, 1);

-- reports
INSERT INTO reports (reporter_id, target_comment_id, reason)
VALUES 
  (3, 2, 'ネガティブな発言が含まれていると感じた');
