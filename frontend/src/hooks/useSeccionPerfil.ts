/* [205A-3] Hook view-model de SeccionPerfil.
 * Extrae la lógica local del componente para que la vista quede en JSX puro.
 * Maneja selector de avatar y confirmación de cambio de email vía modal. */
import {useState} from 'react';
import type {ChangeEvent, FormEvent} from 'react';

import {usePerfil} from './usePerfil';

interface RetornoUseSeccionPerfil extends ReturnType<typeof usePerfil> {
    modalCambioEmailAbierto: boolean;
    confirmacionEmail: string;
    errorConfirmacionEmail: string | null;
    emailObjetivo: string;
    cerrarModalCambioEmail: () => void;
    setConfirmacionEmail: (valor: string) => void;
    limpiarErrorConfirmacionEmail: () => void;
    handleSubmitPerfil: (e: FormEvent) => Promise<void>;
    confirmarCambioEmail: (e: FormEvent) => Promise<void>;
    alSeleccionarArchivo: (e: ChangeEvent<HTMLInputElement>) => void;
}

function normalizarEmail(valor: string): string {
    return valor.trim().toLowerCase();
}

export function useSeccionPerfil(): RetornoUseSeccionPerfil {
    const perfilState = usePerfil();
    const [modalCambioEmailAbierto, setModalCambioEmailAbierto] = useState(false);
    const [confirmacionEmail, setConfirmacionEmailState] = useState('');
    const [errorConfirmacionEmail, setErrorConfirmacionEmail] = useState<string | null>(null);

    const emailActualNormalizado = normalizarEmail(perfilState.perfil?.email ?? '');
    const emailObjetivo = perfilState.estado.email.trim();
    const emailObjetivoNormalizado = normalizarEmail(emailObjetivo);
    const requiereConfirmacionEmail = Boolean(emailObjetivo)
        && emailObjetivoNormalizado !== emailActualNormalizado;

    const cerrarModalCambioEmail = () => {
        setModalCambioEmailAbierto(false);
        setConfirmacionEmailState('');
        setErrorConfirmacionEmail(null);
    };

    const limpiarErrorConfirmacionEmail = () => {
        setErrorConfirmacionEmail(null);
    };

    const setConfirmacionEmail = (valor: string) => {
        setConfirmacionEmailState(valor);
    };

    const handleSubmitPerfil = async (e: FormEvent) => {
        e.preventDefault();
        setErrorConfirmacionEmail(null);
        if (requiereConfirmacionEmail) {
            setModalCambioEmailAbierto(true);
            setConfirmacionEmailState('');
            return;
        }
        await perfilState.guardarPerfil();
    };

    const confirmarCambioEmail = async (e: FormEvent) => {
        e.preventDefault();
        if (normalizarEmail(confirmacionEmail) !== emailObjetivoNormalizado) {
            setErrorConfirmacionEmail(
                'El correo escrito no coincide con el nuevo correo que quieres guardar.'
            );
            return;
        }

        const guardadoOk = await perfilState.guardarPerfil();
        if (guardadoOk) {
            cerrarModalCambioEmail();
        }
    };

    const alSeleccionarArchivo = (e: ChangeEvent<HTMLInputElement>) => {
        const archivo = e.target.files?.[0];
        if (archivo) {
            void perfilState.handleSubirAvatar(archivo);
            /* Limpiar para permitir subir el mismo archivo de nuevo */
            e.target.value = '';
        }
    };

    return {
        ...perfilState,
        modalCambioEmailAbierto,
        confirmacionEmail,
        errorConfirmacionEmail,
        emailObjetivo,
        cerrarModalCambioEmail,
        setConfirmacionEmail,
        limpiarErrorConfirmacionEmail,
        handleSubmitPerfil,
        confirmarCambioEmail,
        alSeleccionarArchivo,
    };
}