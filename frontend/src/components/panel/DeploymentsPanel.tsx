/* [215A-14] Panel admin de despliegues reales — vista tabla minimalista.
 * [225A-3] La fila vive en DeploymentRow para mantener este panel compacto. */

import React from 'react';
import {useTranslation} from 'react-i18next';
import {Server} from 'lucide-react';
import {useDeploymentsPanel} from '../../hooks/useDeploymentsPanel';
import {DeploymentRow, getDeploymentPanelErrorMessage} from './DeploymentRow';
import './VpsPanel.css';

export const DeploymentsPanel: React.FC = () => {
    const {t} = useTranslation();
    const {deployments, isLoading, error} = useDeploymentsPanel();

    if (isLoading) {
        return (
            <div className="vpsLoading">
                <Server size={28} strokeWidth={1.2} />
                <p>{t('panel.deployments.loading', 'Consultando despliegues reales de todas las VPS...')}</p>
            </div>
        );
    }

    if (error) {
        return <div className="vpsError"><p>{getDeploymentPanelErrorMessage(error, t)}</p></div>;
    }

    if (deployments.length === 0) {
        return (
            <div className="vpsVacio">
                <Server size={36} strokeWidth={1.2} />
                <p>{t('panel.deployments.empty', 'No se encontraron despliegues reales en ninguna VPS configurada')}</p>
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
                            <th className="infraEncabezado">{t('panel.deployments.col_name', 'Nombre')}</th>
                            <th className="infraEncabezado">{t('panel.deployments.col_status', 'Estado')}</th>
                            <th className="infraEncabezado">{t('panel.deployments.col_plan', 'Plan')}</th>
                            <th className="infraEncabezado">{t('panel.deployments.col_user', 'Usuario')}</th>
                            <th className="infraEncabezado infraEncabezado--recurso">{t('panel.deployments.col_cpu', 'CPU')}</th>
                            <th className="infraEncabezado infraEncabezado--recurso">{t('panel.deployments.col_ram', 'RAM')}</th>
                            <th className="infraEncabezado infraEncabezado--recurso">{t('panel.deployments.col_disk', 'Disco')}</th>
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