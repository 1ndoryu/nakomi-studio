import React from 'react';
import {Button} from './Button';
import {navegar} from '../../navegacionSPA';
import './SeccionCta.css';

export interface SeccionCtaProps {
    titulo?: string;
    descripcion?: string | string[];
    textoBotonPrimario?: string;
    linkBotonPrimario?: string;
    textoBotonSecundario?: string;
    linkBotonSecundario?: string;
    onBotonPrimarioClick?: () => void;
    onBotonSecundarioClick?: () => void;
    className?: string;
}

export const SeccionCta: React.FC<SeccionCtaProps> = ({titulo, descripcion, textoBotonPrimario, linkBotonPrimario, textoBotonSecundario, linkBotonSecundario, onBotonPrimarioClick, onBotonSecundarioClick, className = ''}) => {
    const parrafos = Array.isArray(descripcion) ? descripcion : descripcion ? [descripcion] : [];

    const handlePrimarioClick = () => {
        if (onBotonPrimarioClick) {
            onBotonPrimarioClick();
        } else if (linkBotonPrimario) {
            navegar(linkBotonPrimario);
        }
    };

    const handleSecundarioClick = () => {
        if (onBotonSecundarioClick) {
            onBotonSecundarioClick();
        } else if (linkBotonSecundario) {
            navegar(linkBotonSecundario);
        }
    };

    return (
        <section className={`seccionCta ${className}`}>
            <div className="ctaContenedor">
                <div className="ctaTexto">
                    {titulo && <h2 className="ctaTitulo">{titulo}</h2>}
                    {parrafos.map((texto) => (
                        <p key={texto}>{texto}</p>
                    ))}
                </div>
                {(textoBotonPrimario || textoBotonSecundario) && (
                    <div className="ctaBotones">
                        {textoBotonPrimario && (
                            <Button variante="primario" tamano="mediano" className="ctaAccionPrimaria" onClick={handlePrimarioClick}>
                                {textoBotonPrimario}
                            </Button>
                        )}
                        {textoBotonSecundario && (
                            <Button variante="outline" tamano="mediano" className="ctaAccionSecundaria" onClick={handleSecundarioClick}>
                                {textoBotonSecundario}
                            </Button>
                        )}
                    </div>
                )}
            </div>
        </section>
    );
};
