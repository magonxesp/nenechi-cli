-- Your SQL goes here
CREATE TABLE `wallpaper_metadata`(
	`id` VARCHAR NOT NULL PRIMARY KEY,
	`pixiv_illustration_id` VARCHAR,
	`tags` JSON NOT NULL,
	`aspect_ratio` VARCHAR NOT NULL
);
