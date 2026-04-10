/* [104A-32] Componente de burbuja de mensaje extraído de SeccionChat.tsx.
 * Renderiza rich messages (invoices, service cards, order cards, archivos)
 * además de texto plano. El backend guarda message_type y metadata JSONB
 * en chat_messages. Hasta 104A-32, solo ChatWidget renderizaba estos tipos. */

import React from 'react';
import {FileText, Palette, Package} from 'lucide-react';
import {SENDER_LABELS, type ChatMessage} from '../../api/chat';
import OptimizedImage from '../ui/OptimizedImage';
import {DEFAULT_PROFILE_AVATAR} from '../../hooks/useCurrentProfile';

export function MessageBubble({message}: {message: ChatMessage}) {
    const isAi = message.sender_type === 'ai';
    const isOwn = message.sender_type === 'client';
    const displayName = message.sender_display_name
        || SENDER_LABELS[message.sender_type]
        || message.sender_type;

    return (
        <div className={`chatBurbuja ${isOwn ? 'chatBurbujaPropia' : ''} ${isAi ? 'chatBurbujaIA' : ''}`}>
            <div className="chatBurbujaHeader">
                {/* [074A-32] Avatar del sender — usa DEFAULT_PROFILE_AVATAR como fallback */}
                <OptimizedImage
                    className="chatBurbujaAvatar"
                    src={message.sender_avatar_url || DEFAULT_PROFILE_AVATAR}
                    alt={displayName}
                />
                <span className={resolveSenderToneClass(message.sender_type)}>{displayName}</span>
                <span className="chatBurbujaHora">
                    {new Date(message.created_at).toLocaleTimeString('es', {
                        hour: '2-digit',
                        minute: '2-digit',
                    })}
                </span>
            </div>
            <div className="chatBurbujaContenido">{renderMessageContent(message)}</div>
        </div>
    );
}

function renderMessageContent(msg: ChatMessage): React.ReactNode {
    const fileUrl = (msg.metadata?.file_url as string) || '';
    const fileName = (msg.metadata?.file_name as string) || 'archivo';

    switch (msg.message_type) {
        case 'image':
            return (
                <div className="chatBurbujaRich">
                    <a href={fileUrl} target="_blank" rel="noopener noreferrer">
                        <OptimizedImage
                            src={fileUrl}
                            alt={fileName}
                            className="chatBurbujaRichImagen"
                            loading="lazy"
                        />
                    </a>
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        case 'audio':
            return (
                <div className="chatBurbujaRich">
                    <audio controls preload="metadata" className="chatBurbujaRichAudio">
                        <source src={fileUrl} />
                    </audio>
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        case 'file':
            return (
                <div className="chatBurbujaRich">
                    <a href={fileUrl} target="_blank" rel="noopener noreferrer"
                        className="chatBurbujaRichArchivoLink">
                        <FileText size={14} /> {fileName}
                    </a>
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        case 'invoice': {
            const payUrl = (msg.metadata?.payment_url as string) || '';
            const amountCents = (msg.metadata?.amount_cents as number) || 0;
            const currency = (msg.metadata?.currency as string) || 'usd';
            const description = (msg.metadata?.description as string) || '';
            const status = (msg.metadata?.status as string) || '';
            const amountFormatted = (amountCents / 100).toFixed(2);
            const isPaid = status === 'paid';
            const isOpen = status === 'open' || status === '' || status === 'draft';

            return (
                <div className="chatBurbujaRich chatBurbujaFactura">
                    <div className="chatBurbujaFacturaHeader">
                        <span className="chatBurbujaFacturaTitulo">Factura</span>
                        {isPaid && <span className="chatBurbujaFacturaPagada">Pagada</span>}
                        {isOpen && <span className="chatBurbujaFacturaPendiente">Pendiente</span>}
                    </div>
                    <p className="chatBurbujaFacturaMonto">
                        ${amountFormatted} {currency.toUpperCase()}
                    </p>
                    {description && (
                        <p className="chatBurbujaFacturaDesc">{description}</p>
                    )}
                    {payUrl && !isPaid && (
                        <a href={payUrl} target="_blank" rel="noopener noreferrer"
                            className="chatBurbujaFacturaBoton">
                            Pagar ahora
                        </a>
                    )}
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        }
        case 'service_card': {
            const title = (msg.metadata?.title as string) || '';
            const desc = (msg.metadata?.description as string) || '';
            const basePriceCents = (msg.metadata?.base_price_cents as number) || 0;
            const priceFormatted = (basePriceCents / 100).toFixed(2);

            return (
                <div className="chatBurbujaRich chatBurbujaServicio">
                    <div className="chatBurbujaServicioHeader">
                        <Palette size={16} />
                        <span className="chatBurbujaServicioTitulo">{title}</span>
                    </div>
                    {desc && <p className="chatBurbujaServicioDesc">{desc}</p>}
                    <p className="chatBurbujaServicioPrecio">Desde ${priceFormatted} USD</p>
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        }
        case 'order_card': {
            const orderNumber = (msg.metadata?.order_number as string) || '';
            const serviceTitle = (msg.metadata?.service_title as string) || '';
            const orderStatus = (msg.metadata?.status as string) || '';

            return (
                <div className="chatBurbujaRich chatBurbujaPedido">
                    <div className="chatBurbujaPedidoHeader">
                        <Package size={16} />
                        <span className="chatBurbujaPedidoTitulo">Pedido #{orderNumber}</span>
                    </div>
                    {serviceTitle && <p className="chatBurbujaPedidoServicio">{serviceTitle}</p>}
                    {orderStatus && <p className="chatBurbujaPedidoEstado">Estado: {orderStatus}</p>}
                    {msg.content && <p className="chatBurbujaRichCaption">{msg.content}</p>}
                </div>
            );
        }
        default:
            return <>{msg.content}</>;
    }
}

export function resolveSenderToneClass(senderType: string) {
    switch (senderType) {
        case 'admin':
            return 'chatRemitente chatRemitente--admin';
        case 'employee':
            return 'chatRemitente chatRemitente--employee';
        case 'client':
            return 'chatRemitente chatRemitente--client';
        case 'ai':
            return 'chatRemitente chatRemitente--ai';
        case 'visitor':
            return 'chatRemitente chatRemitente--visitor';
        default:
            return 'chatRemitente chatRemitente--neutral';
    }
}
