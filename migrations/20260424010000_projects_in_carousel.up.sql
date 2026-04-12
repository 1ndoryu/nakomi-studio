-- [124A-PROJ] Agrega campo in_carousel a projects para controlar independientemente
-- si aparece en el carrusel del hero. Por defecto true (aparece en ambos).
ALTER TABLE projects ADD COLUMN IF NOT EXISTS in_carousel BOOLEAN NOT NULL DEFAULT true;
