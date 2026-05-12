-- Test user seed for E2E testing
-- Email: test@example.com
-- Password: password123

USE archivio_parlante_x;

-- Test user (password hash for 'password123' with bcrypt cost 12)
INSERT INTO ap_users (email, full_name, password_hash, role, active)
VALUES (
  'test@example.com',
  'Test User',
  '$2y$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5L9TLGY.L1DGa',
  'user',
  1
) ON DUPLICATE KEY UPDATE email=email;

-- Test knowledge base
INSERT INTO ap_knowledge_bases (name, description, user_id)
SELECT 'Test KB', 'E2E Testing Knowledge Base', id
FROM ap_users WHERE email = 'test@example.com'
LIMIT 1
ON DUPLICATE KEY UPDATE name=name;

-- Show created records
SELECT 'User created:' as status;
SELECT id, email, full_name, role FROM ap_users WHERE email = 'test@example.com';

SELECT 'Knowledge base created:' as status;
SELECT kb.id, kb.name, kb.description, u.email as owner_email
FROM ap_knowledge_bases kb
JOIN ap_users u ON kb.user_id = u.id
WHERE u.email = 'test@example.com';
