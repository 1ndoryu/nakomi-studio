/* [215A-14] Panel admin de despliegues reales — vista tabla minimalista.
 * [225A-3] La fila vive en DeploymentRow para mantener este panel compacto. */

import React from 'react';
import {Server} from 'lucide-react';
import {useVps2DeploymentsPanel} from '../../hooks/useVps2DeploymentsPanel';
import {DeploymentRow, getDeploymentPanelErrorMessage} from './DeploymentRow';
import './VpsPanel.css';

export const Vps2DeploymentsPanel: React.FC = () => {
    const {deployments, isLoading, error} = useVps2DeploymentsPanel();

    if (isLoading) {
        return (
            <div className="vpsLoading">
                <Server size={28} strokeWidth={1.2} />
                <p>Consultando despliegues reales de todas las VPS...</p>
            </div>
        );
    }

    if (error) {
        return <div className="vpsError"><p>{getDeploymentPanelErrorMessage(error)}</p></div>;
    }

    if (deployments.length === 0) {
        return (
            <div className="vpsVacio">
                <Server size={36} strokeWidth={1.2} />
                <p>No se encontraron despliegues reales en ninguna VPS configurada</p>
            </div>
        );
    }

    return (
        <div className="vpsContenedor">
            <div className="infraTablaWrapper">
                <table className="infraTabla">
                    <thead>
                        <tr>
                            <th className="infraEncabezado infraEncabezado--tipo" />
                            <th className="infraEncabezado">Nombre</th>
                            <th className="infraEncabezado">Estado</th>
                            <th className="infraEncabezado">Plan</th>
                            <th className="infraEncabezado">Usuario</th>
                            <th className="infraEncabezado infraEncabezado--recurso">CPU</th>
                            <th className="infraEncabezado infraEncabezado--recurso">RAM</th>
                            <th className="infraEncabezado infraEncabezado--recurso">Disco</th>
                            <th className="infraEncabezado infraEncabezado--acciones" />
                        </tr>
                    </thead>
                    <tbody>
                        {deployments.map(deployment => (
                            <DeploymentRow key={deployment.uuid} deployment={deployment} />
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
};