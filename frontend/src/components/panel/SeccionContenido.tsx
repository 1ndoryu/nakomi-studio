/* [074A-7] Sección "Contenido" del panel admin — CMS editorial.
 * Sub-tabs laterales: Servicios | Blog | Proyectos | Equipo.
 * [084A-22] Refactorizado: cada sub-tab es su propio componente (SRP).
 * SeccionContenido solo orquesta la navegación entre sub-tabs. */
import React, {useState, useEffect} from 'react';
import {Briefcase, PenTool, FolderOpen, Users} from 'lucide-react';
import {Button} from '../ui/Button';
import {SubTabServicios} from './SubTabServicios';
import {SubTabBlog} from './SubTabBlog';
import {SubTabProyectos} from './SubTabProyectos';
import {SubTabEquipo} from './SubTabEquipo';
import './SeccionContenido.css';

type SubTab = 'servicios' | 'blog' | 'proyectos' | 'equipo';

interface SubTabConfig {
    id: SubTab;
    label: string;
    icono: React.ElementType;
}

const SUB_TABS: SubTabConfig[] = [
    {id: 'servicios', label: 'Servicios', icono: Briefcase},
    {id: 'blog', label: 'Blog', icono: PenTool},
    {id: 'proyectos', label: 'Proyectos', icono: FolderOpen},
    {id: 'equipo', label: 'Equipo', icono: Users},
];

/* [084A-9] Clave de localStorage para persistir sub-tab CMS entre recargas.
 * [154A-12b] Cambiado de sessionStorage a localStorage. */
const CMS_SUBTAB_KEY = 'panel-cms-subtab';
const VALID_SUBTABS: SubTab[] = SUB_TABS.map(t => t.id);

export const SeccionContenido: React.FC = () => {
    /* [084A-9] Restaurar sub-tab desde localStorage si es válida */
    const [subTab, setSubTab] = useState<SubTab>(() => {
        const stored = localStorage.getItem(CMS_SUBTAB_KEY) as SubTab | null;
        if (stored && VALID_SUBTABS.includes(stored)) return stored;
        return 'servicios';
    });

    /* [084A-9] Persistir sub-tab en localStorage */
    useEffect(() => {
        localStorage.setItem(CMS_SUBTAB_KEY, subTab);
    }, [subTab]);

    return (
        <div className="contenidoContenedor">
            <div className="contenidoSubTabs">
                {SUB_TABS.map(tab => {
                    const Icono = tab.icono;
                    return (
                        <Button
                            key={tab.id}
                            type="button"
                            variante="texto"
                            className={`contenidoSubTab ${subTab === tab.id ? 'contenidoSubTab--activo' : ''}`}
                            onClick={() => setSubTab(tab.id)}
                        >
                            <Icono size={16} />
                            {tab.label}
                        </Button>
                    );
                })}
            </div>

            <div className="contenidoPanel">
                {subTab === 'servicios' && <SubTabServicios />}
                {subTab === 'blog' && <SubTabBlog />}
                {subTab === 'proyectos' && <SubTabProyectos />}
                {subTab === 'equipo' && <SubTabEquipo />}
            </div>
        </div>
    );
};
