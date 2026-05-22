/* [064A-32] Planes de WordPress hosting.
 * [084A-10] 3 planes base del catálogo comercial.
 * [204A-1] Fallback alineado con pricing persistido: $2.48, $4.13 y $6.19.
 * [114A-5] Especializacion WordPress: pre-instalado, WP-CLI.
 * [215A-4][215A-9] Beneficios obligatorios y planes genericos sin prometer WooCommerce. */

import {incluida} from './tipos';
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
            incluida('Tráfico ilimitado'),
            incluida('Free temporary domain'),
            incluida('Certificado SSL incluido'),
            incluida('Free CDN'),
            incluida('Backups semanales'),
            incluida('WP-CLI + SSH'),
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
            incluida('Tráfico ilimitado'),
            incluida('Free temporary domain'),
            incluida('Certificado SSL incluido'),
            incluida('Free CDN'),
            incluida('Backups diarios'),
            incluida('WP-CLI + SSH'),
            incluida('Staging environment'),
        ],
    },
    {
        id: 'hosting-ecommerce',
        nombre: 'Avanzado',
        precio: '$6.19',
        periodo: '/mes',
        descripcion: 'WordPress administrado de mayor capacidad para sitios con más contenido, tráfico y caché avanzada.',
        destacado: false,
        ctaTexto: 'Elegir Avanzado',
        ctaLink: '#',
        stripeModo: 'subscription',
        caracteristicas: [
            incluida('WordPress pre-instalado'),
            incluida('50 GB almacenamiento SSD'),
            incluida('Tráfico ilimitado'),
            incluida('Free temporary domain'),
            incluida('Certificado SSL incluido'),
            incluida('Free CDN'),
            incluida('Backups diarios + semanales'),
            incluida('Soporte prioritario 24/7'),
            incluida('WP-CLI + SSH'),
            incluida('Caché avanzada WordPress'),
        ],
    },
];
