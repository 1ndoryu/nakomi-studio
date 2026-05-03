/* [104A-28] Modales de cancelación y reporte de problemas para OrdenDetalle.
 * Cancelar: empleados deben escribir razón obligatoria; clientes confirman directamente.
 * Reportar: textarea para describir el problema, envía a backend vía apiReportProblem. */
import {useState} from 'react';
import {Button} from '../ui/Button';
import {Modal} from '../ui/Modal';
import {Textarea} from '../ui/Textarea';

interface OrderDetailModalsProps {
    orderNumber: number;
    isEmployee: boolean;
    modalCancelarAbierto: boolean;
    modalReportarAbierto: boolean;
    cancelando: boolean;
    reportando: boolean;
    reportError: string | null;
    reportExito: boolean;
    onCerrarCancelar: () => void;
    onConfirmarCancelacion: (reason?: string) => void;
    onCerrarReportar: () => void;
    onEnviarReporte: (reason: string) => void;
}

export function OrderDetailModals({
    orderNumber,
    isEmployee,
    modalCancelarAbierto,
    modalReportarAbierto,
    cancelando,
    reportando,
    reportError,
    reportExito,
    onCerrarCancelar,
    onConfirmarCancelacion,
    onCerrarReportar,
    onEnviarReporte,
}: OrderDetailModalsProps) {
    const [cancelReason, setCancelReason] = useState('');
    const [reportReason, setReportReason] = useState('');

    const handleCerrarCancelar = () => {
        setCancelReason('');
        onCerrarCancelar();
    };

    const handleConfirmarCancelacion = () => {
        onConfirmarCancelacion(isEmployee ? cancelReason : undefined);
    };

    const handleCerrarReportar = () => {
        setReportReason('');
        onCerrarReportar();
    };

    const handleEnviarReporte = () => {
        if (reportReason.trim().length < 10) return;
        onEnviarReporte(reportReason.trim());
    };

    return (
        <>
            {/* Modal cancelar orden */}
            <Modal
                abierto={modalCancelarAbierto}
                onCerrar={() => {
                    if (!cancelando) handleCerrarCancelar();
                }}
                className="ordenDetalleModal"
            >
                <div className="ordenDetalleModalContenido">
                    <p className="modalTexto">
                        Esta acción no se puede deshacer. La orden #{orderNumber} quedará cancelada.
                    </p>
                    {isEmployee && (
                        <Textarea
                            className="ordenDetalleModalTextarea"
                            placeholder="Razón de la cancelación (obligatorio para empleados)"
                            value={cancelReason}
                            onChange={e => setCancelReason(e.target.value)}
                            rows={3}
                            disabled={cancelando}
                        />
                    )}
                    <div className="modalAcciones">
                        <Button variante="outline" tamano="pequeno" type="button"
                            onClick={handleCerrarCancelar} disabled={cancelando}>
                            Volver
                        </Button>
                        <Button variante="secundario" tamano="pequeno" type="button"
                            onClick={handleConfirmarCancelacion}
                            disabled={cancelando || (isEmployee && cancelReason.trim().length < 5)}>
                            {cancelando ? 'Cancelando...' : 'Sí, cancelar'}
                        </Button>
                    </div>
                </div>
            </Modal>

            {/* Modal reportar problema */}
            <Modal
                abierto={modalReportarAbierto}
                onCerrar={() => {
                    if (!reportando) handleCerrarReportar();
                }}
                className="ordenDetalleModal"
            >
                <div className="ordenDetalleModalContenido">
                    {reportExito ? (
                        <>
                            <p className="modalTexto modalTextoExito">
                                Tu reporte fue enviado. El equipo lo revisará pronto.
                            </p>
                            <div className="modalAcciones">
                                <Button variante="outline" tamano="pequeno" type="button"
                                    onClick={handleCerrarReportar}>
                                    Cerrar
                                </Button>
                            </div>
                        </>
                    ) : (
                        <>
                            <p className="modalTexto">
                                Describe el problema con la orden #{orderNumber}. El equipo de soporte lo revisará.
                            </p>
                            <Textarea
                                className="ordenDetalleModalTextarea"
                                placeholder="Describe el problema (mínimo 10 caracteres)"
                                value={reportReason}
                                onChange={e => setReportReason(e.target.value)}
                                rows={4}
                                disabled={reportando}
                            />
                            {reportError && (
                                <p className="ordenDetalleModalError">{reportError}</p>
                            )}
                            <div className="modalAcciones">
                                <Button variante="outline" tamano="pequeno" type="button"
                                    onClick={handleCerrarReportar} disabled={reportando}>
                                    Cancelar
                                </Button>
                                <Button variante="primario" tamano="pequeno" type="button"
                                    onClick={handleEnviarReporte}
                                    disabled={reportando || reportReason.trim().length < 10}>
                                    {reportando ? 'Enviando...' : 'Enviar reporte'}
                                </Button>
                            </div>
                        </>
                    )}
                </div>
            </Modal>
        </>
    );
}