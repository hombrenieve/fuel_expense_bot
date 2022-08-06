create table config (
	username VARCHAR(32) PRIMARY KEY,
	chatId INT NOT NULL,
	payLimit DOUBLE DEFAULT 180.00
);

create table counts (
	txDate DATE NOT NULL,
	username VARCHAR(32),
	quantity DOUBLE,
	PRIMARY KEY(txDate, username),
	FOREIGN KEY(username) REFERENCES config(username)
);