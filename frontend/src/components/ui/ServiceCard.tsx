import React from 'react';
import {useTranslation} from 'react-i18next';
import {ArrowRight} from 'lucide-react';
import {Badge} from './Badge';
import {AdminOverlay} from './AdminOverlay';
import {Servicio} from '../../types/servicios';
import {spaClick} from '../../navegacionSPA';
import './ServiceCard.css';

interface ServiceCardProps {
    servicio: Servicio;
    variant?: 'simple' | 'detailed'; /* 'simple' para Home/Relacionados, 'detailed' para Pagina Servicios */
}

/* [064A-64] Titulo y descripcion traducidos via content translations */
export const ServiceCard: React.FC<ServiceCardProps> = ({servicio, variant = 'simple'}) => {
    const {t} = useTranslation();
    const titulo = t(`content.services.${servicio.id}.titulo`, servicio.titulo);
    const descripcion = t(`content.services.${servicio.id}.descripcion`, servicio.descripcion);

    if (variant === 'detailed') {
        return (
            <AdminOverlay contentType="service" itemId={servicio.adminId || servicio.id}>
                <a href={servicio.link} className="serviceCard detailed" onClick={e => spaClick(e, servicio.link)}>
                    <div className="cardImageWrapper">
                        <img src={servicio.imagen} alt={titulo} className="cardImage" loading="lazy" />
                        <div className="cardOverlay" />
                    </div>
                    <div className="cardContent">
                        <h3 className="cardTitle">{titulo}</h3>
                        <p className="cardDescription">{descripcion}</p>
                        {servicio.categorias && (
                            <div className="cardTags">
                                {servicio.categorias.map((cat) => (
                                    <Badge key={cat} label={cat} />
                                ))}
                            </div>
                        )}
                    </div>
                </a>
            </AdminOverlay>
        );
    }

    /* Variante simple (Home / Relacionados) */
    return (
        <AdminOverlay contentType="service" itemId={servicio.adminId || servicio.id}>
            <a href={servicio.link} className="serviceCard simple" onClick={e => spaClick(e, servicio.link)}>
                <div className="simpleImageWrapper">
                    <img src={servicio.imagen} alt={titulo} className="simpleImage" loading="lazy" />
                </div>
                <div className="simpleContent">
                    <h3 className="simpleTitle">{titulo}</h3>
                    <ArrowRight className="simpleArrow" />
                </div>
            </a>
        </AdminOverlay>
    );
};
