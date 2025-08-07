INSERT INTO user_login_tokens(user_id, token, expire_after)
VALUES (:user_id, :token, datetime('now', '+30 day'))