/* [164A-9] Banner de solicitud de cancelación pendiente.
 * Cliente ve la solicitud del empleado con Accept/Reject.
 * Empleado ve un aviso de que su solicitud está pendiente. */
import React from 'react';
import { AlertTriangle } from 'lucide-react';
import { Button } from '../ui/Button';
import type { CancellationRequestResponse } from '../../api/wallet';
import './CancellationBanner.css';

interface CancellationBannerProps {
    request: CancellationRequestResponse;
    isClient: boolean;
    responding: boolean;
    onAccept: () => void;
    onReject: () => void;
}

export const CancellationBanner: React.FC<CancellationBannerProps> = ({
    request,
    isClient,
    responding,
    onAccept,
    onReject,
}) => {
    return (
        <div className="cancelBanner">
            <div className="cancelBannerIcono">
                <AlertTriangle size={20} />
            </div>
            <div className="cancelBannerContenido">
                <p className="cancelBannerTitulo">Solicitud de cancelación pendiente</p>
                <p className="cancelBannerRazon">{request.reason}</p>

                {isClient && (
                    <div className="cancelBannerAcciones">
                        <Button
                            variante="primario"
                            tamano="pequeno"
                            onClick={onAccept}
                            disabled={responding}
                        >
                            {responding ? 'Procesando...' : 'Aceptar cancelación'}
                        </Button>
                        <Button
                            variante="texto"
                            tamano="pequeno"
                            onClick={onReject}
                            disabled={responding}
                        >
                            Rechazar
                        </Button>
                    </div>
                )}

                {!isClient && (
                    <p className="cancelBannerEstado">
                        Esperando respuesta del cliente...
                    </p>
                )}
            </div>
        </div>
    );
};
