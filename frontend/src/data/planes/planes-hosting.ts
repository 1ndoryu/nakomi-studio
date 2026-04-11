/* [064A-32] Planes de WordPress hosting.
 * [084A-10] 3 planes: Básico $5, Pro $10, E-commerce $15.
 * [114A-5] Especializacion WordPress: pre-instalado, WP-CLI, WooCommerce.
 * CTA abre chat (provisioning manual por ahora — automático cuando infra esté lista). */

import {incluida, noIncluida} from './tipos';
import type {PlanServicio} from './tipos';

export const PLANES_HOSTING: PlanServicio[] = [
    {
        id: 'hosting-basico',
        nombre: 'Básico',
        precio: '$5',
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
            incluida('1 dominio incluido'),
            incluida('WP-CLI vía SSH'),
            noIncluida('CDN global'),
            noIncluida('Staging environment'),
        ],
    },
    {
        id: 'hosting-pro',
        nombre: 'Pro',
        precio: '$10',
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
            incluida('3 dominios incluidos'),
            incluida('CDN global'),
            incluida('WP-CLI vía SSH'),
            incluida('Staging environment'),
        ],
    },
    {
        id: 'hosting-ecommerce',
        nombre: 'E-commerce',
        precio: '$15',
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
            incluida('5 dominios incluidos'),
            incluida('Soporte prioritario 24/7'),
            incluida('WP-CLI vía SSH'),
            incluida('Caché avanzada WordPress'),
        ],
    },
];
