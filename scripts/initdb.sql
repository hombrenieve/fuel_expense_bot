create table config (
	username VARCHAR(32) PRIMARY KEY,
	chatId INT NOT NULL,
	payLimit DOUBLE DEFAULT 180.00
);

create table counts (
	id MEDIUMINT NOT NULL AUTO_INCREMENT,
	txDate DATE NOT NULL,
	username VARCHAR(32),
	quantity DOUBLE,
	PRIMARY KEY(id),
	FOREIGN KEY(username) REFERENCES config(username)
);