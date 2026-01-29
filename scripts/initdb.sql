create table config (
	username VARCHAR(32) PRIMARY KEY,
	chatId BIGINT NOT NULL,
	payLimit DECIMAL(10,2) DEFAULT 210.00
);

create table counts (
	id MEDIUMINT NOT NULL AUTO_INCREMENT,
	txDate DATE NOT NULL,
	username VARCHAR(32),
	quantity DECIMAL(10,2),
	PRIMARY KEY(id),
	FOREIGN KEY(username) REFERENCES config(username),
	UNIQUE KEY unique_user_date (username, txDate)
);