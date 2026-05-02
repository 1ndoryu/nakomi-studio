/* [304A-1] Sección Infraestructura del panel admin.
 * Agrupa los paneles de infraestructura real que antes vivían como tabs dentro
 * de SeccionHosting: despliegues Coolify (VPS2) y servidores Contabo.
 * Separado en su propia entrada del sidebar porque no es "hosting de clientes". */

import React, {useState} from 'react';
import {Button} from '../ui/Button';
import {VpsPanel} from './VpsPanel';
import {Vps2DeploymentsPanel} from './Vps2DeploymentsPanel';
import './SeccionInfraestructura.css';

type TabInfra = 'despliegues' | 'servidores';

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
                    Despliegues VPS2
                </Button>
                <Button
                    type="button"
                    variante="texto"
                    className={`infraTab ${tab === 'servidores' ? 'infraTab--activa' : ''}`}
                    onClick={() => setTab('servidores')}
                >
                    Contabo VPS
                </Button>
            </div>

            {tab === 'despliegues' ? (
                <Vps2DeploymentsPanel />
            ) : (
                <VpsPanel />
            )}
        </div>
    );
};
