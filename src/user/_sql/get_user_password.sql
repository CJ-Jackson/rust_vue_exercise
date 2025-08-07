SELECT id, password
FROM users
WHERE username = :username
LIMIT 1;