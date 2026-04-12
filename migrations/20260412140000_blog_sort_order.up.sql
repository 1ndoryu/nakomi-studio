/* [124A-CMS10] Añadir sort_order a blog_posts para reordenamiento por arrastre */
ALTER TABLE blog_posts ADD COLUMN sort_order INT NOT NULL DEFAULT 0;

/* Inicializar orden basado en fecha de creación */
WITH numbered AS (
    SELECT id, ROW_NUMBER() OVER (ORDER BY created_at) - 1 AS rn
    FROM blog_posts
)
UPDATE blog_posts SET sort_order = numbered.rn
FROM numbered WHERE blog_posts.id = numbered.id;
