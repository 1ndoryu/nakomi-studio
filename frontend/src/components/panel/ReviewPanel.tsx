/* [044A-38 Fase 8] Panel de review dentro del detalle de orden.
 * Cliente: crear review (rating estrellas + comentario).
 * Empleado: responder review existente.
 * Todos: ver review y respuesta si existen. */

import { useState } from 'react';
import { Star, MessageSquare, Loader2 } from 'lucide-react';
import { useOrderReview, useReviews } from '../../hooks/useReviews';
import { useReviewForm } from '../../hooks/useReviewForm';
import { Textarea } from '../ui/Textarea';
import { Button } from '../ui/Button';
import './ReviewPanel.css';

interface ReviewPanelProps {
    orderId: string;
    effectiveRole: string;
}

export function ReviewPanel({ orderId, effectiveRole }: ReviewPanelProps) {
    const { review, cargando } = useOrderReview(orderId);
    const { responderReview } = useReviews();
    const form = useReviewForm(orderId);

    const [respuesta, setRespuesta] = useState('');
    const [respondiendo, setRespondiendo] = useState(false);

    const handleResponder = async () => {
        if (!review || !respuesta.trim()) return;
        setRespondiendo(true);
        try {
            await responderReview.mutateAsync({
                reviewId: review.id,
                response: respuesta.trim(),
            });
            setRespuesta('');
        } finally {
            setRespondiendo(false);
        }
    };

    if (cargando) {
        return (
            <div className="reviewCargando">
                <Loader2 className="reviewSpinner" size={20} />
            </div>
        );
    }

    /* Si ya existe review, mostrarla */
    if (review) {
        return (
            <div className="reviewContenedor">
                <h4 className="reviewTitulo">
                    <Star size={16} /> Review
                </h4>
                <div className="reviewEstrellas">
                    {[1, 2, 3, 4, 5].map((n) => (
                        <Star
                            key={n}
                            size={18}
                            className={n <= review.rating ? 'reviewEstrellaLlena' : 'reviewEstrellaVacia'}
                        />
                    ))}
                    <span className="reviewRatingNumero">{review.rating}/5</span>
                </div>
                {review.comment && (
                    <p className="reviewComentario">{review.comment}</p>
                )}
                <p className="reviewFecha">
                    {new Date(review.created_at).toLocaleDateString('es-ES', {
                        day: 'numeric', month: 'short', year: 'numeric',
                    })}
                </p>

                {/* Respuesta del empleado */}
                {review.employee_response && (
                    <div className="reviewRespuesta">
                        <p className="reviewRespuestaLabel">
                            <MessageSquare size={14} /> Respuesta del profesional
                        </p>
                        <p className="reviewRespuestaTexto">{review.employee_response}</p>
                    </div>
                )}

                {/* Formulario de respuesta para empleado */}
                {effectiveRole === 'employee' && !review.employee_response && (
                    <div className="reviewFormResponder">
                        <Textarea
                            className="reviewTextarea"
                            placeholder="Escribe tu respuesta..."
                            value={respuesta}
                            onChange={(e) => setRespuesta(e.target.value)}
                            rows={3}
                        />
                        <Button
                            variante="texto"
                            className="reviewAccionResponder"
                            onClick={() => void handleResponder()}
                            disabled={respondiendo || !respuesta.trim()}
                            type="button"
                        >
                            {respondiendo ? 'Enviando…' : 'Responder'}
                        </Button>
                    </div>
                )}
            </div>
        );
    }

    /* Si no hay review y es el cliente, mostrar formulario */
    if (effectiveRole === 'client') {
        return (
            <div className="reviewContenedor">
                <h4 className="reviewTitulo">
                    <Star size={16} /> Deja tu valoración
                </h4>
                <div className="reviewEstrellas reviewEstrellasInput">
                    {[1, 2, 3, 4, 5].map((n) => (
                        <Button
                            variante="texto"
                            key={n}
                            className="reviewEstrellaControl"
                            onClick={() => form.setRating(n)}
                            type="button"
                        >
                            <Star
                                size={24}
                                className={n <= form.rating ? 'reviewEstrellaLlena' : 'reviewEstrellaVacia'}
                            />
                        </Button>
                    ))}
                </div>
                <Textarea
                    className="reviewTextarea"
                    placeholder="Cuéntanos tu experiencia (opcional)..."
                    value={form.comment}
                    onChange={(e) => form.setComment(e.target.value)}
                    rows={3}
                />
                <Button
                    className="reviewAccionEnviar"
                    onClick={() => void form.enviarReview()}
                    disabled={form.enviando || form.rating < 1}
                    type="button"
                >
                    {form.enviando ? 'Enviando…' : 'Enviar valoración'}
                </Button>
            </div>
        );
    }

    return null;
}
