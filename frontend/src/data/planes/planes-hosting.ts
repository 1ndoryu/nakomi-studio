/* [064A-32] Planes de WordPress hosting.
 * [084A-10] 3 planes base del catálogo comercial.
 * [204A-1] Fallback alineado con pricing persistido: $2.48, $4.13 y $6.19.
 * [114A-5] Especializacion WordPress: pre-instalado, WP-CLI, WooCommerce.
 * CTA abre chat (provisioning manual por ahora — automático cuando infra esté lista). */

import {incluida, noIncluida} from './tipos';
import type {PlanServicio} from './tipos';

export const PLANES_HOSTING: PlanServicio[] = [
    {
        id: 'hosting-basico',
        nombre: 'Básico',
        precio: '$2.48',
        periodo: '/mes',
        descripcion: 'WordPress pre-instalado, ideal para sitios personales y landing pages con tráfico moderado.',
        destacado: false,
        ctaTexto: 'Comenzar',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: [
            incluida('WordPress pre-instalado'),
            incluida('5 GB almacenamiento SSD'),
            incluida('SSL gratuito'),
            incluida('Backups semanales'),
            incluida('WP-CLI vía SSH'),
            noIncluida('CDN global'),
            noIncluida('Staging environment'),
        ],
    },
    {
        id: 'hosting-pro',
        nombre: 'Pro',
        precio: '$4.13',
        periodo: '/mes',
        descripcion: 'WordPress optimizado para negocios en crecimiento que necesitan rendimiento y fiabilidad.',
        destacado: true,
        ctaTexto: 'Elegir Pro',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: [
            incluida('WordPress pre-instalado'),
            incluida('20 GB almacenamiento SSD'),
            incluida('SSL gratuito'),
            incluida('Backups diarios'),
            incluida('CDN global'),
            incluida('WP-CLI vía SSH'),
            incluida('Staging environment'),
        ],
    },
    {
        id: 'hosting-ecommerce',
        nombre: 'E-commerce',
        precio: '$6.19',
        periodo: '/mes',
        descripcion: 'WordPress + WooCommerce optimizado para tiendas online con alto tráfico y transacciones.',
        destacado: false,
        ctaTexto: 'Elegir E-commerce',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: [
            incluida('WordPress + WooCommerce'),
            incluida('50 GB almacenamiento SSD'),
            incluida('SSL gratuito'),
            incluida('Backups diarios + snapshots'),
            incluida('Soporte prioritario 24/7'),
            incluida('WP-CLI vía SSH'),
            incluida('Caché avanzada WordPress'),
        ],
    },
];
