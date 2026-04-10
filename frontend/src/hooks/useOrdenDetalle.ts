/* [104A-28] Hook para OrdenDetalle: estado y lógica del componente.
 * Maneja checkout, menú contextual, modales de cancelación, reporte y delegación. */
import {useState, useCallback} from 'react';
import {apiReportProblem} from '../api/problems';
import type {OrderResponse} from '../api/orders';

interface CheckoutState {
    amountCents: number;
    phaseNumber?: number;
}

export function useOrdenDetalle(
    order: OrderResponse,
    onCancelar: (id: string, reason?: string) => Promise<void>,
) {
    const [checkout, setCheckout] = useState<CheckoutState | null>(null);
    const [menuAbierto, setMenuAbierto] = useState(false);
    const [modalCancelarAbierto, setModalCancelarAbierto] = useState(false);
    const [modalReportarAbierto, setModalReportarAbierto] = useState(false);
    const [reportando, setReportando] = useState(false);
    const [reportError, setReportError] = useState<string | null>(null);
    const [reportExito, setReportExito] = useState(false);

    const confirmarCancelacion = async (reason?: string) => {
        await onCancelar(order.id, reason);
        setModalCancelarAbierto(false);
    };

    const enviarReporte = useCallback(async (reason: string) => {
        setReportando(true);
        setReportError(null);
        try {
            await apiReportProblem(order.id, {reason});
            setReportExito(true);
        } catch (err: unknown) {
            const msg = (err as {response?: {data?: {message?: string}}})
                ?.response?.data?.message || 'Error al reportar';
            setReportError(msg);
        } finally {
            setReportando(false);
        }
    }, [order.id]);

    const cerrarReportar = useCallback(() => {
        setModalReportarAbierto(false);
        setReportError(null);
        setReportExito(false);
    }, []);

    const abrirCheckout = (amountCents: number, phaseNumber?: number) => {
        setCheckout({amountCents, phaseNumber});
    };

    const cerrarCheckout = () => setCheckout(null);

    return {
        checkout,
        menuAbierto,
        setMenuAbierto,
        modalCancelarAbierto,
        setModalCancelarAbierto,
        modalReportarAbierto,
        setModalReportarAbierto,
        confirmarCancelacion,
        enviarReporte,
        reportando,
        reportError,
        reportExito,
        cerrarReportar,
        abrirCheckout,
        cerrarCheckout,
    };
}
