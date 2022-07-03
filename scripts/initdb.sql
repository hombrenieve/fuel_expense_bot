create table counts (
	username VARCHAR(32) PRIMARY KEY,
	payLimit DOUBLE DEFAULT 180.00,
	paid DOUBLE
);