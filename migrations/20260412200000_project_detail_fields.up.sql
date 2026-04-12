ALTER TABLE projects ADD COLUMN detail_title TEXT;
ALTER TABLE projects ADD COLUMN use_first_gallery_image BOOLEAN NOT NULL DEFAULT false;
