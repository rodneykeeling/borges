CREATE TYPE status AS ENUM ('unread', 'read', 'reading');
ALTER TABLE book ADD COLUMN status status DEFAULT 'unread' NOT NULL;

UPDATE book SET status = 'read' WHERE title = 'Collected Fictions';
