/* [124A-PROJ1] Revert: convertir gallery de [{"url":"...","layout":"..."}] a ["url"] */
UPDATE projects
SET gallery = (
    SELECT COALESCE(jsonb_agg(elem->>'url'), '[]'::jsonb)
    FROM jsonb_array_elements(gallery) AS elem
)
WHERE jsonb_typeof(gallery) = 'array'
  AND jsonb_array_length(gallery) > 0
  AND jsonb_typeof(gallery->0) = 'object';
