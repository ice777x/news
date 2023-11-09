CREATE TABLE feeds (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    link TEXT UNIQUE,
    description TEXT,
    author TEXT,
    image TEXT,
    content TEXT,
    published TEXT
)