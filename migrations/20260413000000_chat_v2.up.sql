/* [P-2 Chatbot v2] Tablas y columnas para chat enriquecido.
 * visitor_profiles: memoria persistente de visitantes.
 * chat_attachments: archivos adjuntos en mensajes.
 * message_type + metadata en chat_messages: mensajes ricos (facturas, servicios, acciones).
 * ai_intermediary en orders: toggle IA por pedido. */

/* Perfiles de visitantes — memoria persistente por identidad */
CREATE TABLE visitor_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    visitor_id VARCHAR(64) UNIQUE NOT NULL,
    email VARCHAR(255),
    user_id UUID REFERENCES users(id),
    display_name VARCHAR(100),
    context_summary TEXT,
    preferences JSONB DEFAULT '{}',
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_sessions INTEGER NOT NULL DEFAULT 0,
    ip_addresses TEXT[] DEFAULT '{}',
    device_fingerprints TEXT[] DEFAULT '{}'
);
CREATE INDEX idx_visitor_profiles_email ON visitor_profiles(email) WHERE email IS NOT NULL;
CREATE INDEX idx_visitor_profiles_user ON visitor_profiles(user_id) WHERE user_id IS NOT NULL;

/* Adjuntos de mensajes (imágenes, archivos, audio) */
CREATE TABLE chat_attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    ai_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chat_attachments_message ON chat_attachments(message_id);

/* Mensajes ricos: tipo + metadatos estructurados */
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS message_type VARCHAR(20) NOT NULL DEFAULT 'text';
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS metadata JSONB;

/* IA intermediaria en pedidos */
ALTER TABLE orders ADD COLUMN IF NOT EXISTS ai_intermediary_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS ai_summary TEXT;
