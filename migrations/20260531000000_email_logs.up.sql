/* [311A-1] Tabla de trazabilidad de correos enviados.
 * Registra cada envío SMTP: destinatario, asunto, plantilla, estado.
 * Non-fatal: si el logging falla, el email igual se entrega. */
CREATE TABLE IF NOT EXISTS email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    to_email TEXT NOT NULL,
    subject TEXT NOT NULL,
    template TEXT NOT NULL,
    reference_type TEXT,
    reference_id UUID,
    status TEXT NOT NULL DEFAULT 'sent',
    error_msg TEXT,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_logs_created_at ON email_logs (created_at DESC);
CREATE INDEX idx_email_logs_template ON email_logs (template);
CREATE INDEX idx_email_logs_status ON email_logs (status);
CREATE INDEX idx_email_logs_reference ON email_logs (reference_type, reference_id);
