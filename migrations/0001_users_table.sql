CREATE TABLE users (

  id BYTEA UNIQUE PRIMARY KEY,
  name VARCHAR(255) UNIQUE NOT NULL,
  login VARCHAR(255) UNIQUE NOT NULL,
  password VARCHAR(255) UNIQUE NOT NULL

);
