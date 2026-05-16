/* sentinel-disable-file limite-lineas */
/* [064A-64] Traducciones de contenido de negocio para 3 idiomas (es, en, ja).
 * Incluye: servicios, equipo, planes, soluciones, hosting.
 * Registrado en i18n.ts como recurso adicional del namespace 'translation'.
 * Justificación sentinel-disable: archivo de datos de contenido, no lógica. */

export interface ContentTranslations {
    services: Record<string, { titulo: string; descripcion: string; skills?: Record<string, { titulo: string; descripcion: string }> }>;
    team: Record<string, { bio: string; cargo: string }>;
    plans: Record<string, { nombre: string; descripcion: string; cta: string; features: string[] }>;
    solutions: Record<string, { titulo: string; descripcion: string; etiqueta: string; features?: Record<string, { titulo: string; desc: string }> }>;
    projects: Record<string, { descripcion: string; cliente: string }>;
}

export const contentEs: ContentTranslations = {
    services: {
        '1': {
            titulo: 'Diseño de Sitios Web',
            descripcion: 'Sitios web a medida con diseño original, rendimiento optimizado y código limpio. Desde landing pages hasta plataformas complejas.',
            skills: {
                '1': {titulo: 'Responsive Design', descripcion: 'Diseño adaptativo para todos los dispositivos.'},
                '2': {titulo: 'Performance', descripcion: 'Optimización de velocidad de carga y Core Web Vitals.'},
                '3': {titulo: 'SEO On-page', descripcion: 'Estructura semántica y meta tags optimizados.'},
                '4': {titulo: 'CMS Integration', descripcion: 'Integración con WordPress, headless CMS o soluciones propias.'},
            },
        },
        '2': {
            titulo: 'Desarrollo de Aplicaciones',
            descripcion: 'Software a medida: apps web, móviles y de escritorio. Arquitectura sólida, APIs robustas y experiencia de usuario cuidada.',
            skills: {
                '1': {titulo: 'React / React Native', descripcion: 'Frontend web y apps móviles cross-platform.'},
                '2': {titulo: 'API Development', descripcion: 'APIs REST con Rust (Axum) o Node.js.'},
                '3': {titulo: 'Cloud Architecture', descripcion: 'Infraestructura escalable con deploy automatizado.'},
                '4': {titulo: 'Testing & QA', descripcion: 'Suite de tests automatizados para confiabilidad.'},
            },
        },
        '3': {titulo: 'Agentes de IA', descripcion: 'Asistentes inteligentes, chatbots contextuales y automatización con modelos de lenguaje. Integración con tu flujo de trabajo existente.'},
        '4': {titulo: 'Identidad de Marca', descripcion: 'Identidad visual completa: logo, paleta, tipografía y guidelines. Una marca coherente que comunica tu esencia.'},
        '5': {titulo: 'E-commerce', descripcion: 'Tiendas online con catálogo, checkout optimizado, pasarelas de pago y panel de gestión. Enfocadas en conversión.'},
    },
    team: {
        wan: {bio: 'Fundadora y directora creativa con visión estratégica para soluciones digitales de alto impacto.', cargo: 'CEO & Founder'},
        anthony: {bio: 'Ingeniero de software principal, especializado en arquitecturas escalables y rendimiento.', cargo: 'Lead Developer'},
        misael: {bio: 'Ingeniero DevOps enfocado en la automatización, despliegue continuo y estabilidad de infraestructura.', cargo: 'DevOps Engineer'},
    },
    plans: {
        'web-basico': {nombre: 'Basico', descripcion: 'Ideal para landing pages y presencia online basica.', cta: 'Comenzar', features: ['1 pagina principal', 'Diseno responsive', 'Formulario de contacto', 'Hosting 3 meses incluido', 'Blog integrado', 'SEO avanzado', 'Panel administrable']},
        'web-medio': {nombre: 'Medio', descripcion: 'Sitio multi-pagina con blog y SEO basico incluido.', cta: 'Elegir plan', features: ['Hasta 3 paginas', 'Diseno responsive', 'Formulario de contacto', 'Hosting 6 meses incluido', 'Blog integrado', 'SEO on-page basico', 'Panel administrable']},
        'web-avanzado': {nombre: 'Avanzado', descripcion: 'Sitio completo con CMS, blog y optimizacion SEO.', cta: 'Elegir plan', features: ['Hasta 5 paginas', 'Diseno responsive premium', 'Formulario de contacto', 'Hosting 6 meses incluido', 'Blog integrado', 'SEO on-page completo', 'Panel administrable (CMS)']},
        'apps-basico': {nombre: 'Basico', descripcion: 'MVP o aplicacion sencilla con funcionalidades core.', cta: 'Comenzar', features: ['1 plataforma (web o movil)', 'Hasta 3 funcionalidades', 'Autenticacion basica', 'API REST simple', 'Panel de administracion', 'Integraciones externas', 'Soporte post-lanzamiento']},
        'apps-medio': {nombre: 'Medio', descripcion: 'App completa con panel de admin y autenticacion avanzada.', cta: 'Elegir plan', features: ['1 plataforma (web o movil)', 'Hasta 5 funcionalidades', 'Auth + roles de usuario', 'API REST completa', 'Panel de administracion', '1 integracion externa', 'Soporte post-lanzamiento']},
        'apps-avanzado': {nombre: 'Avanzado', descripcion: 'App completa multi-plataforma con backend robusto.', cta: 'Elegir plan', features: ['Web + Movil (React Native)', 'Hasta 8 funcionalidades', 'Auth + roles de usuario', 'API REST + GraphQL', 'Panel de administracion', '2 integraciones externas', '1 mes soporte post-lanzamiento']},
        'ia-basico': {nombre: 'Basico', descripcion: 'Agente simple para tareas de automatizacion basica.', cta: 'Comenzar', features: ['1 agente configurado', 'Hasta 1,000 interacciones/mes', 'Integracion con 1 plataforma', 'Respuestas predefinidas', 'Aprendizaje continuo', 'Analisis de datos', 'API personalizada']},
        'ia-avanzado': {nombre: 'Avanzado', descripcion: 'Agente inteligente con IA generativa y analisis.', cta: 'Elegir plan', features: ['Hasta 3 agentes', 'Interacciones ilimitadas', 'Multi-plataforma', 'IA generativa (GPT/Gemini)', 'Aprendizaje continuo', 'Dashboard de analisis', 'API personalizada']},
        'ia-personalizado': {nombre: 'Personalizado', descripcion: 'Solucion de IA a la medida de tu negocio.', cta: 'Hablar con nosotros', features: ['Agentes ilimitados', 'Modelo fine-tuned', 'Integracion total', 'Datos propietarios', 'SLA garantizado', 'Soporte dedicado 24/7', 'Consultoria IA estrategica']},
        'branding-basico': {nombre: 'Basico', descripcion: 'Logo y paleta de colores para tu marca.', cta: 'Comenzar', features: ['Diseno de logo (3 propuestas)', 'Paleta de colores', 'Tipografia principal', 'Archivo en formatos digitales', 'Manual de marca', 'Papeleria corporativa', 'Redes sociales kit']},
        'branding-medio': {nombre: 'Medio', descripcion: 'Identidad visual con manual de marca y papeleria.', cta: 'Elegir plan', features: ['Diseno de logo (4 propuestas)', 'Paleta de colores extendida', 'Sistema tipografico', 'Manual de marca (PDF)', 'Papeleria corporativa', 'Kit redes sociales', 'Iconografia personalizada']},
        'branding-avanzado': {nombre: 'Avanzado', descripcion: 'Identidad visual completa con manual de marca.', cta: 'Elegir plan', features: ['Diseno de logo (5 propuestas)', 'Paleta de colores extendida', 'Sistema tipografico completo', 'Manual de marca (PDF)', 'Papeleria corporativa', 'Kit redes sociales', 'Iconografia personalizada']},
        'ecommerce-basico': {nombre: 'Basico', descripcion: 'Tienda online funcional con lo esencial para vender.', cta: 'Comenzar', features: ['Hasta 50 productos', 'Pasarela de pago (Stripe)', 'Carrito de compras', 'Diseno responsive', 'Gestion de inventario', 'Integraciones avanzadas', 'Marketing automatizado']},
        'ecommerce-avanzado': {nombre: 'Avanzado', descripcion: 'Tienda completa con inventario y marketing integrado.', cta: 'Elegir plan', features: ['Productos ilimitados', 'Multi-pasarela de pago', 'Gestion de inventario', 'Panel de analytics', 'Email marketing integrado', 'Cupones y descuentos', 'Marketplace multi-vendor']},
        'ecommerce-personalizado': {nombre: 'Personalizado', descripcion: 'Solucion e-commerce a medida para operaciones complejas.', cta: 'Hablar con nosotros', features: ['Arquitectura personalizada', 'Integraciones ERP/CRM', 'Multi-idioma y multi-moneda', 'Marketplace multi-vendor', 'Analytics avanzado', 'Soporte prioritario 24/7', 'Migracion de datos']},
        'seo-basico': {nombre: 'Basico', descripcion: 'Auditoria y optimizacion SEO inicial.', cta: 'Comenzar', features: ['Auditoria SEO completa', 'Optimizacion de 5 paginas', 'Configuracion Search Console', 'Reporte mensual basico', 'Link building', 'Contenido optimizado', 'SEO local']},
        'seo-medio': {nombre: 'Medio', descripcion: 'SEO con contenido optimizado y link building basico.', cta: 'Elegir plan', features: ['Estrategia SEO integral', 'Optimizacion de 15 paginas', '2 articulos SEO/mes', 'Link building basico', 'SEO local + Google My Business', 'Reportes semanales', 'Analisis de competencia']},
        'seo-avanzado': {nombre: 'Avanzado', descripcion: 'Estrategia SEO completa con contenido y links.', cta: 'Elegir plan', features: ['Estrategia SEO integral', 'Optimizacion paginas ilimitadas', '4 articulos SEO/mes', 'Link building activo', 'SEO local + Google My Business', 'Reportes semanales', 'Analisis de competencia']},
        'mkt-basico': {nombre: 'Basico', descripcion: 'Gestion basica de redes sociales y ads.', cta: 'Comenzar', features: ['Gestion de 2 redes sociales', '8 publicaciones/mes', '1 campana de ads/mes', 'Reporte mensual', 'Email marketing', 'Estrategia de contenidos', 'Influencer marketing']},
        'mkt-medio': {nombre: 'Medio', descripcion: 'Marketing con email, contenido y ads avanzados.', cta: 'Elegir plan', features: ['Gestion de 3 redes sociales', '15 publicaciones/mes', 'Campanas ads ilimitadas', 'Email marketing (newsletters)', 'Estrategia de contenidos', 'Reportes semanales', 'A/B testing campanas']},
        'mkt-avanzado': {nombre: 'Avanzado', descripcion: 'Estrategia de marketing digital integral.', cta: 'Elegir plan', features: ['Gestion de 4+ redes sociales', '20 publicaciones/mes', 'Campanas ads ilimitadas', 'Email marketing + automatizacion', 'Estrategia de contenidos', 'Reportes semanales', 'A/B testing + Influencer marketing']},
        'hosting-basico': {nombre: 'Básico', descripcion: 'WordPress pre-instalado, ideal para sitios personales y landing pages con tráfico moderado.', cta: 'Comenzar', features: ['WordPress pre-instalado', '5 GB almacenamiento SSD', 'SSL gratuito', 'Backups semanales', 'Soporte por chat', 'WP-CLI vía SSH', 'Actualizaciones WordPress']},
        'hosting-pro': {nombre: 'Pro', descripcion: 'WordPress optimizado para negocios en crecimiento que necesitan rendimiento y fiabilidad.', cta: 'Elegir Pro', features: ['WordPress pre-instalado', '20 GB almacenamiento SSD', 'SSL gratuito', 'Backups diarios', 'CDN global', 'WP-CLI vía SSH', 'Staging environment']},
        'hosting-ecommerce': {nombre: 'E-commerce', descripcion: 'WordPress + WooCommerce optimizado para tiendas online con alto tráfico y transacciones.', cta: 'Elegir E-commerce', features: ['WordPress + WooCommerce', '50 GB almacenamiento SSD', 'SSL gratuito', 'Backups diarios + snapshots', 'Soporte prioritario 24/7', 'WP-CLI vía SSH', 'Caché avanzada WordPress']},
        'hosting-custom': {nombre: 'A Medida', descripcion: 'WordPress con infraestructura personalizada para proyectos con necesidades específicas.', cta: 'Consultar', features: ['WordPress pre-instalado', '100+ GB almacenamiento', 'SSL gratuito', 'Backups bajo demanda', 'Soporte dedicado', 'WP-CLI vía SSH', 'SLA personalizado']},
    },
    solutions: {
        hosting: {titulo: 'WordPress Hosting', descripcion: 'Hosting especializado en WordPress con rendimiento optimizado, WP-CLI, backups automáticos y soporte experto para que tu sitio nunca se detenga.', etiqueta: 'Desde $2.48/mes', features: {
            performance: {titulo: 'WordPress Optimizado', desc: 'Servidores SSD configurados específicamente para WordPress con caché y PHP optimizado.'},
            security: {titulo: 'Seguridad WordPress', desc: 'SSL gratuito, firewall, backups automáticos, hardening SSH y monitoreo 24/7.'},
            uptime: {titulo: '99.9% Uptime', desc: 'Infraestructura redundante con failover automático para máxima disponibilidad.'},
            cdn: {titulo: 'CDN Global', desc: 'Red de distribución de contenido para velocidad óptima desde cualquier ubicación.'},
            managed: {titulo: 'WordPress Administrado', desc: 'Actualizaciones de WordPress, plugins, parches de seguridad y optimizaciones sin que toques un servidor.'},
            support: {titulo: 'WP-CLI & Soporte', desc: 'Acceso SSH con WP-CLI incluido y equipo técnico experto en WordPress.'},
        }},
        vps: {titulo: 'Servidores VPS', descripcion: 'Control total sobre tu entorno de servidor. Recursos dedicados, acceso root y configuración a medida para proyectos exigentes.', etiqueta: 'Desde $6.88/mes'},
        'agentes-ia': {titulo: 'Agentes de IA', descripcion: 'Automatiza procesos de negocio con agentes inteligentes. Chatbots, asistentes virtuales y flujos automatizados con IA de última generación.', etiqueta: 'Consultar'},
    },
    projects: {
        kamples: {descripcion: 'Plataforma de samples musicales con algoritmo de recomendación, DAW integrado y funcionalidades de red social. Código abierto.', cliente: 'Open Source Platform'},
        mabuhay: {descripcion: 'Web y branding para agencia de viajes en España especializada en destinos asiáticos. Diseño cálido y visual que transmite la esencia de cada destino.', cliente: 'Agencia de Viajes'},
        guillermochatbot: {descripcion: 'Portfolio interactivo con chatbot IA que responde sobre experiencia profesional, proyectos y skills técnicos.', cliente: 'AI Portfolio'},
        task: {descripcion: 'Aplicación de gestión de tareas con enfoque en simplicidad y flujos de trabajo ágiles.', cliente: 'Productivity App'},
        'material-de-padel': {descripcion: 'Tienda online de equipamiento de pádel con catálogo, comparador de palas y blog de contenido deportivo.', cliente: 'E-commerce Deportivo'},
    },
};

export const contentEn: ContentTranslations = {
    services: {
        '1': {
            titulo: 'Website Design',
            descripcion: 'Custom websites with original design, optimized performance, and clean code. From landing pages to complex platforms.',
            skills: {
                '1': {titulo: 'Responsive Design', descripcion: 'Adaptive design for all devices.'},
                '2': {titulo: 'Performance', descripcion: 'Loading speed and Core Web Vitals optimization.'},
                '3': {titulo: 'SEO On-page', descripcion: 'Semantic structure and optimized meta tags.'},
                '4': {titulo: 'CMS Integration', descripcion: 'Integration with WordPress, headless CMS, or custom solutions.'},
            },
        },
        '2': {
            titulo: 'Application Development',
            descripcion: 'Custom software: web, mobile, and desktop apps. Solid architecture, robust APIs, and polished user experience.',
            skills: {
                '1': {titulo: 'React / React Native', descripcion: 'Web frontend and cross-platform mobile apps.'},
                '2': {titulo: 'API Development', descripcion: 'REST APIs with Rust (Axum) or Node.js.'},
                '3': {titulo: 'Cloud Architecture', descripcion: 'Scalable infrastructure with automated deployment.'},
                '4': {titulo: 'Testing & QA', descripcion: 'Automated test suites for reliability.'},
            },
        },
        '3': {titulo: 'AI Agents', descripcion: 'Smart assistants, contextual chatbots, and automation with language models. Integration with your existing workflow.'},
        '4': {titulo: 'Brand Identity', descripcion: 'Complete visual identity: logo, palette, typography, and guidelines. A coherent brand that communicates your essence.'},
        '5': {titulo: 'E-commerce', descripcion: 'Online stores with catalog, optimized checkout, payment gateways, and admin panel. Focused on conversion.'},
    },
    team: {
        wan: {bio: 'Founder and creative director with strategic vision for high-impact digital solutions.', cargo: 'CEO & Founder'},
        anthony: {bio: 'Lead software engineer specializing in scalable architectures and performance.', cargo: 'Lead Developer'},
        misael: {bio: 'DevOps engineer focused on automation, continuous deployment, and infrastructure stability.', cargo: 'DevOps Engineer'},
    },
    plans: {
        'web-basico': {nombre: 'Basic', descripcion: 'Ideal for landing pages and basic online presence.', cta: 'Get Started', features: ['1 main page', 'Responsive design', 'Contact form', '3 months hosting included', 'Integrated blog', 'Advanced SEO', 'Admin panel']},
        'web-medio': {nombre: 'Standard', descripcion: 'Multi-page site with blog and basic SEO included.', cta: 'Choose Plan', features: ['Up to 3 pages', 'Responsive design', 'Contact form', '6 months hosting included', 'Integrated blog', 'Basic on-page SEO', 'Admin panel']},
        'web-avanzado': {nombre: 'Advanced', descripcion: 'Complete site with CMS, blog, and SEO optimization.', cta: 'Choose Plan', features: ['Up to 5 pages', 'Premium responsive design', 'Contact form', '6 months hosting included', 'Integrated blog', 'Full on-page SEO', 'Admin panel (CMS)']},
        'apps-basico': {nombre: 'Basic', descripcion: 'MVP or simple app with core features.', cta: 'Get Started', features: ['1 platform (web or mobile)', 'Up to 3 features', 'Basic authentication', 'Simple REST API', 'Admin panel', 'External integrations', 'Post-launch support']},
        'apps-medio': {nombre: 'Standard', descripcion: 'Complete app with admin panel and advanced auth.', cta: 'Choose Plan', features: ['1 platform (web or mobile)', 'Up to 5 features', 'Auth + user roles', 'Full REST API', 'Admin panel', '1 external integration', 'Post-launch support']},
        'apps-avanzado': {nombre: 'Advanced', descripcion: 'Full multi-platform app with robust backend.', cta: 'Choose Plan', features: ['Web + Mobile (React Native)', 'Up to 8 features', 'Auth + user roles', 'REST API + GraphQL', 'Admin panel', '2 external integrations', '1 month post-launch support']},
        'ia-basico': {nombre: 'Basic', descripcion: 'Simple agent for basic automation tasks.', cta: 'Get Started', features: ['1 configured agent', 'Up to 1,000 interactions/mo', 'Integration with 1 platform', 'Predefined responses', 'Continuous learning', 'Data analysis', 'Custom API']},
        'ia-avanzado': {nombre: 'Advanced', descripcion: 'Intelligent agent with generative AI and analytics.', cta: 'Choose Plan', features: ['Up to 3 agents', 'Unlimited interactions', 'Multi-platform', 'Generative AI (GPT/Gemini)', 'Continuous learning', 'Analytics dashboard', 'Custom API']},
        'ia-personalizado': {nombre: 'Custom', descripcion: 'AI solution tailored to your business.', cta: 'Talk to Us', features: ['Unlimited agents', 'Fine-tuned model', 'Full integration', 'Proprietary data', 'Guaranteed SLA', 'Dedicated 24/7 support', 'Strategic AI consulting']},
        'branding-basico': {nombre: 'Basic', descripcion: 'Logo and color palette for your brand.', cta: 'Get Started', features: ['Logo design (3 proposals)', 'Color palette', 'Primary typography', 'Digital format files', 'Brand manual', 'Corporate stationery', 'Social media kit']},
        'branding-medio': {nombre: 'Standard', descripcion: 'Visual identity with brand manual and stationery.', cta: 'Choose Plan', features: ['Logo design (4 proposals)', 'Extended color palette', 'Typography system', 'Brand manual (PDF)', 'Corporate stationery', 'Social media kit', 'Custom iconography']},
        'branding-avanzado': {nombre: 'Advanced', descripcion: 'Complete visual identity with brand manual.', cta: 'Choose Plan', features: ['Logo design (5 proposals)', 'Extended color palette', 'Full typography system', 'Brand manual (PDF)', 'Corporate stationery', 'Social media kit', 'Custom iconography']},
        'ecommerce-basico': {nombre: 'Basic', descripcion: 'Functional online store with essentials to sell.', cta: 'Get Started', features: ['Up to 50 products', 'Payment gateway (Stripe)', 'Shopping cart', 'Responsive design', 'Inventory management', 'Advanced integrations', 'Automated marketing']},
        'ecommerce-avanzado': {nombre: 'Advanced', descripcion: 'Complete store with inventory and integrated marketing.', cta: 'Choose Plan', features: ['Unlimited products', 'Multi-gateway payment', 'Inventory management', 'Analytics panel', 'Integrated email marketing', 'Coupons and discounts', 'Multi-vendor marketplace']},
        'ecommerce-personalizado': {nombre: 'Custom', descripcion: 'Tailored e-commerce solution for complex operations.', cta: 'Talk to Us', features: ['Custom architecture', 'ERP/CRM integrations', 'Multi-language & multi-currency', 'Multi-vendor marketplace', 'Advanced analytics', 'Priority 24/7 support', 'Data migration']},
        'seo-basico': {nombre: 'Basic', descripcion: 'Initial SEO audit and optimization.', cta: 'Get Started', features: ['Full SEO audit', 'Optimization of 5 pages', 'Search Console setup', 'Basic monthly report', 'Link building', 'Optimized content', 'Local SEO']},
        'seo-medio': {nombre: 'Standard', descripcion: 'SEO with optimized content and basic link building.', cta: 'Choose Plan', features: ['Comprehensive SEO strategy', 'Optimization of 15 pages', '2 SEO articles/month', 'Basic link building', 'Local SEO + Google My Business', 'Weekly reports', 'Competitor analysis']},
        'seo-avanzado': {nombre: 'Advanced', descripcion: 'Complete SEO strategy with content and links.', cta: 'Choose Plan', features: ['Comprehensive SEO strategy', 'Unlimited page optimization', '4 SEO articles/month', 'Active link building', 'Local SEO + Google My Business', 'Weekly reports', 'Competitor analysis']},
        'mkt-basico': {nombre: 'Basic', descripcion: 'Basic social media and ads management.', cta: 'Get Started', features: ['Management of 2 social networks', '8 posts/month', '1 ad campaign/month', 'Monthly report', 'Email marketing', 'Content strategy', 'Influencer marketing']},
        'mkt-medio': {nombre: 'Standard', descripcion: 'Marketing with email, content, and advanced ads.', cta: 'Choose Plan', features: ['Management of 3 social networks', '15 posts/month', 'Unlimited ad campaigns', 'Email marketing (newsletters)', 'Content strategy', 'Weekly reports', 'A/B testing campaigns']},
        'mkt-avanzado': {nombre: 'Advanced', descripcion: 'Comprehensive digital marketing strategy.', cta: 'Choose Plan', features: ['Management of 4+ social networks', '20 posts/month', 'Unlimited ad campaigns', 'Email marketing + automation', 'Content strategy', 'Weekly reports', 'A/B testing + Influencer marketing']},
        'hosting-basico': {nombre: 'Basic', descripcion: 'WordPress pre-installed, ideal for personal sites and landing pages with moderate traffic.', cta: 'Get Started', features: ['WordPress pre-installed', '5 GB SSD storage', 'Free SSL', 'Weekly backups', 'Chat support', 'WP-CLI via SSH', 'WordPress updates']},
        'hosting-pro': {nombre: 'Pro', descripcion: 'Optimized WordPress hosting for growing businesses that need performance and reliability.', cta: 'Choose Pro', features: ['WordPress pre-installed', '20 GB SSD storage', 'Free SSL', 'Daily backups', 'Global CDN', 'WP-CLI via SSH', 'Staging environment']},
        'hosting-ecommerce': {nombre: 'E-commerce', descripcion: 'WordPress + WooCommerce optimized for online stores with high traffic and transactions.', cta: 'Choose E-commerce', features: ['WordPress + WooCommerce', '50 GB SSD storage', 'Free SSL', 'Daily backups + snapshots', 'Priority 24/7 support', 'WP-CLI via SSH', 'Advanced WordPress cache']},
        'hosting-custom': {nombre: 'Custom', descripcion: 'WordPress with personalized infrastructure for projects with specific needs.', cta: 'Get a Quote', features: ['WordPress pre-installed', '100+ GB storage', 'Free SSL', 'On-demand backups', 'Dedicated support', 'WP-CLI via SSH', 'Custom SLA']},
    },
    solutions: {
        hosting: {titulo: 'WordPress Hosting', descripcion: 'Specialized WordPress hosting with optimized performance, WP-CLI, automatic backups, and expert support so your site never stops.', etiqueta: 'From $2.48/mo', features: {
            performance: {titulo: 'WordPress Optimized', desc: 'SSD servers specifically configured for WordPress with optimized caching and PHP.'},
            security: {titulo: 'WordPress Security', desc: 'Free SSL, firewall, automatic backups, SSH hardening, and 24/7 monitoring.'},
            uptime: {titulo: '99.9% Uptime', desc: 'Redundant infrastructure with automatic failover for maximum availability.'},
            cdn: {titulo: 'Global CDN', desc: 'Content delivery network for optimal speed from any location.'},
            managed: {titulo: 'Managed WordPress', desc: 'WordPress updates, plugins, security patches, and optimizations without touching a server.'},
            support: {titulo: 'WP-CLI & Support', desc: 'SSH access with WP-CLI included and expert WordPress technical team.'},
        }},
        vps: {titulo: 'VPS Servers', descripcion: 'Full control over your server environment. Dedicated resources, root access, and custom configuration for demanding projects.', etiqueta: 'From $6.88/mo'},
        'agentes-ia': {titulo: 'AI Agents', descripcion: 'Automate business processes with intelligent agents. Chatbots, virtual assistants, and automated workflows with cutting-edge AI.', etiqueta: 'Contact Us'},
    },
    projects: {
        kamples: {descripcion: 'Music samples platform with recommendation algorithm, integrated DAW, and social network features. Open source.', cliente: 'Open Source Platform'},
        mabuhay: {descripcion: 'Website and branding for a travel agency in Spain specializing in Asian destinations. Warm visual design that conveys the essence of each destination.', cliente: 'Travel Agency'},
        guillermochatbot: {descripcion: 'Interactive portfolio with an AI chatbot that answers about professional experience, projects, and technical skills.', cliente: 'AI Portfolio'},
        task: {descripcion: 'Task management app focused on simplicity and agile workflows.', cliente: 'Productivity App'},
        'material-de-padel': {descripcion: 'Online padel equipment store with catalog, paddle comparison tool, and sports content blog.', cliente: 'Sports E-commerce'},
    },
};

export const contentJa: ContentTranslations = {
    services: {
        '1': {
            titulo: 'ウェブデザイン',
            descripcion: 'オリジナルデザイン、最適化されたパフォーマンス、クリーンなコードによるカスタムウェブサイト。ランディングページから複雑なプラットフォームまで。',
            skills: {
                '1': {titulo: 'レスポンシブデザイン', descripcion: '全デバイス対応のアダプティブデザイン。'},
                '2': {titulo: 'パフォーマンス', descripcion: '読み込み速度とCore Web Vitalsの最適化。'},
                '3': {titulo: 'SEO On-page', descripcion: 'セマンティック構造と最適化されたメタタグ。'},
                '4': {titulo: 'CMS連携', descripcion: 'WordPress、ヘッドレスCMS、またはカスタムソリューションとの連携。'},
            },
        },
        '2': {
            titulo: 'アプリケーション開発',
            descripcion: 'カスタムソフトウェア：Web、モバイル、デスクトップアプリ。堅牢なアーキテクチャ、強力なAPI、洗練されたUX。',
            skills: {
                '1': {titulo: 'React / React Native', descripcion: 'Webフロントエンドとクロスプラットフォームモバイルアプリ。'},
                '2': {titulo: 'API開発', descripcion: 'Rust（Axum）またはNode.jsによるREST API。'},
                '3': {titulo: 'クラウドアーキテクチャ', descripcion: '自動デプロイ対応のスケーラブルなインフラ。'},
                '4': {titulo: 'テスト＆QA', descripcion: '信頼性のための自動テストスイート。'},
            },
        },
        '3': {titulo: 'AIエージェント', descripcion: 'スマートアシスタント、コンテキスト対応チャットボット、言語モデルによる自動化。既存ワークフローとの統合。'},
        '4': {titulo: 'ブランドアイデンティティ', descripcion: '完全なビジュアルアイデンティティ：ロゴ、カラーパレット、タイポグラフィ、ガイドライン。あなたの本質を伝える一貫したブランド。'},
        '5': {titulo: 'Eコマース', descripcion: 'カタログ、最適化されたチェックアウト、決済ゲートウェイ、管理画面を備えたオンラインストア。コンバージョンに焦点を当てた設計。'},
    },
    team: {
        wan: {bio: '高インパクトなデジタルソリューションへの戦略的ビジョンを持つ創設者兼クリエイティブディレクター。', cargo: 'CEO & Founder'},
        anthony: {bio: 'スケーラブルなアーキテクチャとパフォーマンスを専門とするリードソフトウェアエンジニア。', cargo: 'Lead Developer'},
        misael: {bio: '自動化、継続的デプロイメント、インフラの安定性に注力するDevOpsエンジニア。', cargo: 'DevOps Engineer'},
    },
    plans: {
        'web-basico': {nombre: 'ベーシック', descripcion: 'ランディングページと基本的なオンラインプレゼンスに最適。', cta: '始める', features: ['メインページ1つ', 'レスポンシブデザイン', 'お問い合わせフォーム', 'ホスティング3ヶ月付き', 'ブログ', '高度なSEO', '管理画面']},
        'web-medio': {nombre: 'スタンダード', descripcion: 'ブログと基本SEO付きの複数ページサイト。', cta: 'プランを選ぶ', features: ['最大3ページ', 'レスポンシブデザイン', 'お問い合わせフォーム', 'ホスティング6ヶ月付き', 'ブログ', '基本的なオンページSEO', '管理画面']},
        'web-avanzado': {nombre: 'アドバンスド', descripcion: 'CMS、ブログ、SEO最適化を備えた完全なサイト。', cta: 'プランを選ぶ', features: ['最大5ページ', 'プレミアムレスポンシブデザイン', 'お問い合わせフォーム', 'ホスティング6ヶ月付き', 'ブログ', '完全なオンページSEO', '管理画面（CMS）']},
        'apps-basico': {nombre: 'ベーシック', descripcion: 'コア機能を備えたMVPまたはシンプルなアプリ。', cta: '始める', features: ['1プラットフォーム（Webまたはモバイル）', '最大3機能', '基本認証', 'シンプルなREST API', '管理画面', '外部連携', 'ローンチ後サポート']},
        'apps-medio': {nombre: 'スタンダード', descripcion: '管理画面と高度な認証を備えた完全なアプリ。', cta: 'プランを選ぶ', features: ['1プラットフォーム（Webまたはモバイル）', '最大5機能', '認証＋ユーザーロール', '完全なREST API', '管理画面', '外部連携1つ', 'ローンチ後サポート']},
        'apps-avanzado': {nombre: 'アドバンスド', descripcion: '堅牢なバックエンドを備えたマルチプラットフォームアプリ。', cta: 'プランを選ぶ', features: ['Web＋モバイル（React Native）', '最大8機能', '認証＋ユーザーロール', 'REST API＋GraphQL', '管理画面', '外部連携2つ', 'ローンチ後1ヶ月サポート']},
        'ia-basico': {nombre: 'ベーシック', descripcion: '基本的な自動化タスク用のシンプルなエージェント。', cta: '始める', features: ['エージェント1つ', '月間最大1,000インタラクション', '1プラットフォーム連携', '定型応答', '継続学習', 'データ分析', 'カスタムAPI']},
        'ia-avanzado': {nombre: 'アドバンスド', descripcion: '生成AIと分析機能を備えたインテリジェントエージェント。', cta: 'プランを選ぶ', features: ['最大3エージェント', '無制限インタラクション', 'マルチプラットフォーム', '生成AI（GPT/Gemini）', '継続学習', '分析ダッシュボード', 'カスタムAPI']},
        'ia-personalizado': {nombre: 'カスタム', descripcion: 'ビジネスに合わせたオーダーメイドAIソリューション。', cta: 'ご相談ください', features: ['無制限エージェント', 'ファインチューニングモデル', '完全な統合', '独自データ対応', 'SLA保証', '専任24/7サポート', '戦略的AIコンサルティング']},
        'branding-basico': {nombre: 'ベーシック', descripcion: 'ロゴとカラーパレットでブランドを構築。', cta: '始める', features: ['ロゴデザイン（3案）', 'カラーパレット', 'メインタイポグラフィ', 'デジタルフォーマットファイル', 'ブランドマニュアル', '企業ステーショナリー', 'SNSキット']},
        'branding-medio': {nombre: 'スタンダード', descripcion: 'ブランドマニュアルとステーショナリー付きビジュアルアイデンティティ。', cta: 'プランを選ぶ', features: ['ロゴデザイン（4案）', '拡張カラーパレット', 'タイポグラフィシステム', 'ブランドマニュアル（PDF）', '企業ステーショナリー', 'SNSキット', 'カスタムアイコン']},
        'branding-avanzado': {nombre: 'アドバンスド', descripcion: 'ブランドマニュアル付きの完全なビジュアルアイデンティティ。', cta: 'プランを選ぶ', features: ['ロゴデザイン（5案）', '拡張カラーパレット', '完全なタイポグラフィシステム', 'ブランドマニュアル（PDF）', '企業ステーショナリー', 'SNSキット', 'カスタムアイコン']},
        'ecommerce-basico': {nombre: 'ベーシック', descripcion: '販売に必要な基本機能を備えたオンラインストア。', cta: '始める', features: ['最大50商品', '決済ゲートウェイ（Stripe）', 'ショッピングカート', 'レスポンシブデザイン', '在庫管理', '高度な連携', 'マーケティング自動化']},
        'ecommerce-avanzado': {nombre: 'アドバンスド', descripcion: '在庫管理とマーケティング統合を備えた完全なストア。', cta: 'プランを選ぶ', features: ['無制限商品', 'マルチ決済ゲートウェイ', '在庫管理', '分析パネル', 'メールマーケティング連携', 'クーポン＆割引', 'マルチベンダーマーケットプレイス']},
        'ecommerce-personalizado': {nombre: 'カスタム', descripcion: '複雑な運営向けのオーダーメイドEコマースソリューション。', cta: 'ご相談ください', features: ['カスタムアーキテクチャ', 'ERP/CRM連携', '多言語＆多通貨', 'マルチベンダーマーケットプレイス', '高度な分析', '優先24/7サポート', 'データ移行']},
        'seo-basico': {nombre: 'ベーシック', descripcion: '初期SEO監査と最適化。', cta: '始める', features: ['完全なSEO監査', '5ページの最適化', 'Search Console設定', '基本月次レポート', 'リンクビルディング', '最適化コンテンツ', 'ローカルSEO']},
        'seo-medio': {nombre: 'スタンダード', descripcion: '最適化コンテンツと基本リンクビルディング付きSEO。', cta: 'プランを選ぶ', features: ['包括的なSEO戦略', '15ページの最適化', 'SEO記事月2本', '基本リンクビルディング', 'ローカルSEO＋Googleビジネスプロフィール', '週次レポート', '競合分析']},
        'seo-avanzado': {nombre: 'アドバンスド', descripcion: 'コンテンツとリンクを含む完全なSEO戦略。', cta: 'プランを選ぶ', features: ['包括的なSEO戦略', '無制限ページ最適化', 'SEO記事月4本', 'アクティブリンクビルディング', 'ローカルSEO＋Googleビジネスプロフィール', '週次レポート', '競合分析']},
        'mkt-basico': {nombre: 'ベーシック', descripcion: '基本的なSNS管理と広告。', cta: '始める', features: ['SNS2アカウント管理', '月8投稿', '月1広告キャンペーン', '月次レポート', 'メールマーケティング', 'コンテンツ戦略', 'インフルエンサーマーケティング']},
        'mkt-medio': {nombre: 'スタンダード', descripcion: 'メール、コンテンツ、高度な広告を含むマーケティング。', cta: 'プランを選ぶ', features: ['SNS3アカウント管理', '月15投稿', '無制限広告キャンペーン', 'メールマーケティング（ニュースレター）', 'コンテンツ戦略', '週次レポート', 'A/Bテストキャンペーン']},
        'mkt-avanzado': {nombre: 'アドバンスド', descripcion: '包括的なデジタルマーケティング戦略。', cta: 'プランを選ぶ', features: ['SNS4+アカウント管理', '月20投稿', '無制限広告キャンペーン', 'メールマーケティング＋自動化', 'コンテンツ戦略', '週次レポート', 'A/Bテスト＋インフルエンサーマーケティング']},
        'hosting-basico': {nombre: 'ベーシック', descripcion: 'WordPressプリインストール済み。個人サイトやランディングページに最適。', cta: '始める', features: ['WordPressプリインストール', '5GB SSDストレージ', '無料SSL', '週次バックアップ', 'ドメイン1つ付き', 'チャットサポート', 'WP-CLI SSHアクセス', 'WordPressアップデート']},
        'hosting-pro': {nombre: 'プロ', descripcion: '成長中のビジネス向けに最適化されたWordPressホスティング。', cta: 'プロを選ぶ', features: ['WordPressプリインストール', '20GB SSDストレージ', '無料SSL', '日次バックアップ', 'ドメイン3つ付き', 'グローバルCDN', 'WP-CLI SSHアクセス', 'ステージング環境']},
        'hosting-ecommerce': {nombre: 'Eコマース', descripcion: '高トラフィック・高トランザクションのオンラインストア向けにWordPress + WooCommerceを最適化。', cta: 'Eコマースを選ぶ', features: ['WordPress + WooCommerce', '50GB SSDストレージ', '無料SSL', '日次バックアップ＋スナップショット', 'ドメイン5つ付き', '優先24/7サポート', 'WP-CLI SSHアクセス', 'WordPress高度キャッシュ']},
        'hosting-custom': {nombre: 'カスタム', descripcion: '特定のニーズを持つプロジェクト向けのWordPressカスタムインフラ。', cta: 'お見積もり', features: ['WordPressプリインストール', '100GB以上のストレージ', '無料SSL', 'オンデマンドバックアップ', '無制限ドメイン', '専任サポート', 'WP-CLI SSHアクセス', 'カスタムSLA']},
    },
    solutions: {
        hosting: {titulo: 'WordPress ホスティング', descripcion: 'WordPressに特化したホスティング。最適化されたパフォーマンス、WP-CLI、自動バックアップ、専門サポート。', etiqueta: '$2.48/月〜', features: {
            performance: {titulo: 'WordPress最適化', desc: 'WordPress向けに特別設定されたSSDサーバーで最小のロード時間を実現。'},
            security: {titulo: 'WordPressセキュリティ', desc: '無料SSL、ファイアウォール、自動バックアップ、SSHハーデニング、24時間365日モニタリング。'},
            uptime: {titulo: '99.9% アップタイム', desc: '自動フェイルオーバー付きの冗長インフラで最高の可用性を保証。'},
            cdn: {titulo: 'グローバルCDN', desc: 'あらゆる場所から最適な速度を実現するコンテンツ配信ネットワーク。'},
            managed: {titulo: 'WordPressマネージド', desc: 'サーバーに触れることなく、WordPressのアップデート、プラグイン、セキュリティパッチを実施。'},
            support: {titulo: 'WP-CLI & サポート', desc: 'WP-CLI付きSSHアクセスとWordPress専門技術チーム。'},
        }},
        vps: {titulo: 'VPSサーバー', descripcion: 'サーバー環境の完全な制御。専用リソース、rootアクセス、要求の厳しいプロジェクト向けのカスタム設定。', etiqueta: '$6.88/月〜'},
        'agentes-ia': {titulo: 'AIエージェント', descripcion: 'インテリジェントエージェントでビジネスプロセスを自動化。チャットボット、仮想アシスタント、最先端AIによる自動ワークフロー。', etiqueta: 'お問い合わせ'},
    },
    projects: {
        kamples: {descripcion: '推薦アルゴリズム、統合DAW、ソーシャルネットワーク機能を備えた音楽サンプルプラットフォーム。オープンソース。', cliente: 'オープンソースプラットフォーム'},
        mabuhay: {descripcion: 'アジア方面を専門とするスペインの旅行代理店向けのウェブサイトとブランディング。各目的地の魅力を伝える温かみのあるビジュアルデザイン。', cliente: '旅行代理店'},
        guillermochatbot: {descripcion: '職務経験、プロジェクト、技術スキルについて回答するAIチャットボット付きインタラクティブポートフォリオ。', cliente: 'AIポートフォリオ'},
        task: {descripcion: 'シンプルさとアジャイルワークフローに焦点を当てたタスク管理アプリ。', cliente: 'プロダクティビティアプリ'},
        'material-de-padel': {descripcion: 'カタログ、パドル比較ツール、スポーツコンテンツブログを備えたパデル用品オンラインストア。', cliente: 'スポーツEコマース'},
    },
};
