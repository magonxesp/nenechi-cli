-- Your SQL goes here
CREATE TABLE `wallpapers`(
	`id` VARCHAR(32) NOT NULL PRIMARY KEY,
	`pixiv_illustration_id` VARCHAR(255),
	`tags` JSON NOT NULL,
	`aspect_ratio` VARCHAR(255) NOT NULL,
    `path` VARCHAR(255) NOT NULL,
    `file_name` VARCHAR(255) NOT NULL
);
