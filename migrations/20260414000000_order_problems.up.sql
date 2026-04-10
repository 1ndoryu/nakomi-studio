/* [104A-28] Sistema de reporte de problemas en órdenes.
 * Empleados y clientes pueden reportar problemas con razón escrita.
 * Admin gestiona los problemas desde su panel.
 * También añade cancel_reason a orders para que empleados puedan cancelar con justificación. */

CREATE TYPE problem_status AS ENUM ('open', 'in_review', 'resolved', 'dismissed');

CREATE TABLE order_problems (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id    UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reporter_role VARCHAR(20) NOT NULL,
    reason      TEXT NOT NULL,
    status      problem_status NOT NULL DEFAULT 'open',
    admin_response TEXT,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_order_problems_order ON order_problems(order_id);
CREATE INDEX idx_order_problems_status ON order_problems(status);

/* Permit employee cancel with a written reason */
ALTER TABLE orders ADD COLUMN cancel_reason TEXT;
