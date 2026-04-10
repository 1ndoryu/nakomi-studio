ALTER TABLE orders
ADD COLUMN project_description TEXT;

UPDATE orders
SET project_description = client_notes
WHERE client_notes IS NOT NULL
  AND client_notes NOT LIKE 'Pago anticipado:%';