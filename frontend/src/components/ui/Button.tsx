import React from 'react';
import './Button.css';

/* [104A-20] Se eliminaron variantes semánticas de color (exito, peligro, advertencia, info).
 * Solo quedan primario, secundario, outline, texto. Los botones de acción usan
 * secundario (bold) o primario (sutil) según importancia, nunca colores. */
type ButtonVariante = 'primario' | 'secundario' | 'outline' | 'texto';
/* [25AA-1] "icono": botón circular de 32px para un solo icono. */
type ButtonTamano = 'pequeno' | 'mediano' | 'grande' | 'icono';

/* [por que] Con href se renderiza un enlace con el mismo estilo (pagar, descargar):
 * así un enlace con aspecto de botón no necesita una clase de estilo propia. */
type ButtonProps = (React.ButtonHTMLAttributes<HTMLButtonElement> & React.AnchorHTMLAttributes<HTMLAnchorElement>) & {
    variante?: ButtonVariante;
    tamano?: ButtonTamano;
};

/**
 * Componente Boton reutilizable
 * Soporta variantes y tamaños configurables via props. Con href renderiza un enlace.
 */
export const Button: React.FC<ButtonProps> = ({children, className = '', variante = 'primario', tamano = 'mediano', href, ...props}) => {
    const claseVariante = `boton${variante.charAt(0).toUpperCase() + variante.slice(1)}`;
    const claseTamano = `boton${tamano.charAt(0).toUpperCase() + tamano.slice(1)}`;
    const clases = `botonBase ${claseVariante} ${claseTamano} ${className}`;

    if (href !== undefined) {
        return <a className={clases} href={href} {...(props as React.AnchorHTMLAttributes<HTMLAnchorElement>)}>{children}</a>;
    }
    return <button className={clases} {...(props as React.ButtonHTMLAttributes<HTMLButtonElement>)}>{children}</button>;
};
