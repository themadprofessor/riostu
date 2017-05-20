-- Your SQL goes here
CREATE TABLE users (
  id VARCHAR PRIMARY KEY,
  name VARCHAR NOT NULL,
  email VARCHAR NOT NULL
);

CREATE TABLE requests (
  id SERIAL PRIMARY KEY,
  user_id VARCHAR references users(id) NOT NULL,
  amount INTEGER NOT NULL
);

CREATE TABLE tokens (
  user_id VARCHAR PRIMARY KEY REFERENCES users(id),
  token VARCHAR NOT NULL
)