import {useState} from 'react';
import {Grid3X3, ListChecks, Music2, Server} from 'lucide-react';
import {MenuContextual} from '../ui/ContextMenu';

const APPS_NAKOMI = [
    {id: 'tasks', nombre: 'Tasks', url: 'https://task.nakomi.studio', icono: <ListChecks size={18} />},
    {id: 'vps', nombre: 'VPS', url: 'https://vps.nakomi.studio', icono: <Server size={18} />},
    {id: 'kamples', nombre: 'Kamples', url: 'https://kamples.com', icono: <Music2 size={18} />},
];

export function AppLauncher() {
    const [abierto, setAbierto] = useState(false);

    return (
        <MenuContextual
            abierto={abierto}
            onToggle={() => setAbierto(prev => !prev)}
            onCerrar={() => setAbierto(false)}
            ariaLabel="Aplicaciones Nakomi"
            tipo="apps"
            triggerContent={<Grid3X3 size={16} aria-hidden="true" />}
        >
            {/* [105A-38] Launcher de apps junto al avatar: mantiene el patrón MenuContextual
             * y evita reintroducir el botón de chat en la navegación principal. */}
            <div className="menuContextualAppsGrid">
                {APPS_NAKOMI.map(app => (
                    <a
                        key={app.id}
                        className="menuContextualAppsItem"
                        href={app.url}
                        onClick={() => setAbierto(false)}
                    >
                        <span className="menuContextualAppsIcono">{app.icono}</span>
                        <span className="menuContextualAppsNombre">{app.nombre}</span>
                    </a>
                ))}
            </div>
        </MenuContextual>
    );
}