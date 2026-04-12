/* [124A-PROJ1] Migrar gallery de ["url"] a [{"url":"url","layout":"full"}]
 * Solo convierte galerías donde el primer elemento es string (formato viejo).
 * Galerías vacías o ya convertidas no se tocan. */
UPDATE projects
SET gallery = (
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object('url', elem::text, 'layout', 'full')
    ), '[]'::jsonb)
    FROM jsonb_array_elements_text(gallery) AS elem
)
WHERE jsonb_typeof(gallery) = 'array'
  AND jsonb_array_length(gallery) > 0
  AND jsonb_typeof(gallery->0) = 'string';
