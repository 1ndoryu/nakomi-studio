/* [304A-1] Sección Infraestructura del panel admin.
 * Agrupa los paneles de infraestructura real que antes vivían como tabs dentro
 * de SeccionHosting: despliegues Coolify y servidores VPS.
 * Separado en su propia entrada del sidebar porque no es "hosting de clientes". */

import React, {useState} from 'react';
import {Button} from '../ui/Button';
import {VpsPanel} from './VpsPanel';
import {DeploymentsPanel} from './DeploymentsPanel';
import './SeccionInfraestructura.css';

type TabInfra = 'despliegues' | 'vps';

export const SeccionInfraestructura: React.FC = () => {
    const [tab, setTab] = useState<TabInfra>('despliegues');

    return (
        <div className="infraContenedor">
            <div className="infraTabs">
                <Button
                    type="button"
                    variante="texto"
                    className={`infraTab ${tab === 'despliegues' ? 'infraTab--activa' : ''}`}
                    onClick={() => setTab('despliegues')}
                >
                    Despliegues
                </Button>
                <Button
                    type="button"
                    variante="texto"
                    className={`infraTab ${tab === 'vps' ? 'infraTab--activa' : ''}`}
                    onClick={() => setTab('vps')}
                >
                    VPS
                </Button>
            </div>

            {tab === 'despliegues' ? (
                <DeploymentsPanel />
            ) : (
                <VpsPanel />
            )}
        </div>
    );
};
