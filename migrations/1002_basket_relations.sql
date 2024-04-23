ALTER TABLE Basket

ADD CONSTRAINT fk_basket_item
  FOREIGN KEY (itemId)
  REFERENCES Product(id);


ALTER TABLE Basket

ADD CONSTRAINT fk_basket_user
  FOREIGN KEY (userId)
  REFERENCES Users(id);
