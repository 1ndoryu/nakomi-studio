import React, {useRef, useState, useLayoutEffect} from 'react';
import {MoreHorizontal} from 'lucide-react';
import {Button} from './Button';
import './ContextMenu.css';

export interface MenuContextualItem {
    id: string;
    label: string;
    onSelect: () => void;
    disabled?: boolean;
    danger?: boolean;
    icon?: React.ReactNode;
}

interface MenuContextualProps {
    abierto: boolean;
    onToggle: () => void;
    onCerrar: () => void;
    items?: MenuContextualItem[];
    ariaLabel: string;
    className?: string;
    triggerClassName?: string;
    panelClassName?: string;
    itemClassName?: string;
    triggerContent?: React.ReactNode;
    triggerVariante?: 'primario' | 'secundario' | 'outline' | 'texto';
    triggerTamano?: 'pequeno' | 'mediano' | 'grande';
    children?: React.ReactNode;
}

export const MenuContextual: React.FC<MenuContextualProps> = ({
    abierto,
    onToggle,
    onCerrar,
    items = [],
    ariaLabel,
    className = '',
    triggerClassName = '',
    panelClassName = '',
    itemClassName = '',
    triggerContent,
    triggerVariante = 'texto',
    triggerTamano = 'pequeno',
    children,
}) => {
    const contenedorRef = useRef<HTMLDivElement>(null);
    const [abreArriba, setAbreArriba] = useState(false);

    /* Detecta si el panel se saldria por debajo del viewport y ajusta direccion */
    useLayoutEffect(() => {
        if (!abierto || !contenedorRef.current) { return; }
        const rect = contenedorRef.current.getBoundingClientRect();
        const espacioAbajo = window.innerHeight - rect.bottom;
        setAbreArriba(espacioAbajo < 320);
    }, [abierto]);

    const handleBlur = (event: React.FocusEvent<HTMLDivElement>) => {
        const nextTarget = event.relatedTarget as Node | null;
        if (nextTarget && event.currentTarget.contains(nextTarget)) {
            return;
        }
        onCerrar();
    };

    const clasePanel = [
        'menuContextualPanel',
        abreArriba ? 'menuContextualPanel--arriba' : '',
        panelClassName,
    ].filter(Boolean).join(' ');

    return (
        <div ref={contenedorRef} className={`menuContextual ${className}`.trim()} onBlur={handleBlur}>
            <Button
                className={`menuContextualBoton ${triggerClassName}`.trim()}
                onClick={onToggle}
                type="button"
                aria-haspopup="menu"
                aria-expanded={abierto}
                aria-label={ariaLabel}
                variante={triggerVariante}
                tamano={triggerTamano}
            >
                {triggerContent ?? <MoreHorizontal size={18} />}
            </Button>

            {abierto && (
                <div className={clasePanel} role="menu">
                    {children ? (
                        <div className="menuContextualContenido">{children}</div>
                    ) : (
                        items.map(item => (
                            <Button
                                key={item.id}
                                className={[
                                    'menuContextualItem',
                                    item.danger ? 'menuContextualItemDanger' : '',
                                    itemClassName,
                                ].filter(Boolean).join(' ')}
                                onClick={() => {
                                    item.onSelect();
                                    onCerrar();
                                }}
                                disabled={item.disabled}
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                            >
                                {item.icon}
                                <span>{item.label}</span>
                            </Button>
                        ))
                    )}
                </div>
            )}
        </div>
    );
};