create table counts (
	username VARCHAR(32) PRIMARY KEY,
	chatId INT NOT NULL,
	payLimit DOUBLE DEFAULT 180.00,
	autoReset BOOLEAN DEFAULT TRUE,
	paid DOUBLE
);