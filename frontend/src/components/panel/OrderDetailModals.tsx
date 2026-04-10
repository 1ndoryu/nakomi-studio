import {Button} from '../ui/Button';
import {Modal} from '../ui/Modal';

interface OrderDetailModalsProps {
    orderNumber: number;
    modalCancelarAbierto: boolean;
    modalReportarAbierto: boolean;
    cancelando: boolean;
    onCerrarCancelar: () => void;
    onConfirmarCancelacion: () => void;
    onCerrarReportar: () => void;
}

export function OrderDetailModals({
    orderNumber,
    modalCancelarAbierto,
    modalReportarAbierto,
    cancelando,
    onCerrarCancelar,
    onConfirmarCancelacion,
    onCerrarReportar,
}: OrderDetailModalsProps) {
    return (
        <>
            <Modal
                abierto={modalCancelarAbierto}
                onCerrar={() => {
                    if (!cancelando) onCerrarCancelar();
                }}
                className="ordenDetalleModal"
            >
                <div className="ordenDetalleModalContenido">
                    <h3 className="modalTitulo">Cancelar orden</h3>
                    <p className="ordenDetalleModalTexto">
                        Esta acción no se puede deshacer. La orden #{orderNumber} quedará cancelada.
                    </p>
                    <div className="modalAcciones">
                        <Button variante="outline" tamano="pequeno" type="button"
                            onClick={onCerrarCancelar} disabled={cancelando}>
                            Volver
                        </Button>
                        <Button variante="secundario" tamano="pequeno" type="button"
                            onClick={onConfirmarCancelacion} disabled={cancelando}>
                            {cancelando ? 'Cancelando...' : 'Sí, cancelar'}
                        </Button>
                    </div>
                </div>
            </Modal>

            <Modal
                abierto={modalReportarAbierto}
                onCerrar={onCerrarReportar}
                className="ordenDetalleModal"
            >
                <div className="ordenDetalleModalContenido">
                    <h3 className="modalTitulo">Reportar problema</h3>
                    <p className="ordenDetalleModalTexto">
                        El equipo de soporte revisará tu caso y se comunicará contigo. Por ahora, contacta por chat.
                    </p>
                    <div className="modalAcciones">
                        <Button variante="outline" tamano="pequeno" type="button"
                            onClick={onCerrarReportar}>
                            Entendido
                        </Button>
                    </div>
                </div>
            </Modal>
        </>
    );
}