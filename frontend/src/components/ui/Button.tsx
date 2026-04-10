import React from 'react';
import './Button.css';

/* [104A-20] Se eliminaron variantes semánticas de color (exito, peligro, advertencia, info).
 * Solo quedan primario, secundario, outline, texto. Los botones de acción usan
 * secundario (bold) o primario (sutil) según importancia, nunca colores. */
type ButtonVariante = 'primario' | 'secundario' | 'outline' | 'texto';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variante?: ButtonVariante;
    tamano?: 'pequeno' | 'mediano' | 'grande';
}

/**
 * Componente Boton reutilizable
 * Soporta variantes y tamaños configurables via props.
 */
export const Button: React.FC<ButtonProps> = ({children, className = '', variante = 'primario', tamano = 'mediano', ...props}) => {
    const claseVariante = `boton${variante.charAt(0).toUpperCase() + variante.slice(1)}`;
    const claseTamano = `boton${tamano.charAt(0).toUpperCase() + tamano.slice(1)}`;

    return (
        <button className={`botonBase ${claseVariante} ${claseTamano} ${className}`} {...props}>
            {children}
        </button>
    );
};
