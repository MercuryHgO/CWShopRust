ALTER TABLE Comment

ADD CONSTRAINT fk_comment_user
  FOREIGN KEY (userId)
  REFERENCES Users(id);


ALTER TABLE Comment

ADD CONSTRAINT fk_comment_product
  FOREIGN KEY (productId)
  REFERENCES Product(id);
