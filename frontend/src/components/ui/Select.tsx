/* [054A-19] Componente Select reutilizable — wrapper de <select> nativo.
 * Mantiene acceso completo a SelectHTMLAttributes. Similar a Input. */
import React from 'react';
import './Select.css';

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
    variante?: 'default' | 'outline';
}

export const Select: React.FC<SelectProps> = ({className = '', variante = 'default', children, ...props}) => {
    const claseVariante = `select${variante.charAt(0).toUpperCase() + variante.slice(1)}`;
    /* sentinel-disable-next-line select-nativo: este ES el componente base; el native select aquí es intencional */
    return (
        <select className={`selectBase ${claseVariante} ${className}`} {...props}>
            {children}
        </select>
    );
};
