CREATE TABLE comment (

  id BYTEA PRIMARY KEY,
  comment TEXT NOT NULL,
  userId BYTEA NOT NULL,
  productId BYTEA NOT NULL

);
