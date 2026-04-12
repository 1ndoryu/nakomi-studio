/* [124A-SHOW1] Campo showcase_category por proyecto.
 * Permite al usuario definir el título de categoría que aparece
 * en la sección Selected Work del home. Proyectos con el mismo
 * valor se agrupan automáticamente. */
ALTER TABLE projects ADD COLUMN showcase_category TEXT;
