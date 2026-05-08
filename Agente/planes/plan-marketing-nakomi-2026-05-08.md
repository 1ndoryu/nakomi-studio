# Plan maestro de marketing — Nakomi Studio
**Fecha:** 2026-05-08

## Objetivo
Mejorar credibilidad, SEO, posicionamiento y conversion del sitio sin:
- subir precios
- crear paquetes combinados
- meter metricas en los case studies

## Diagnostico resumido
El sitio tiene buena base visual y tecnica, pero arrastraba problemas claros de marketing:
- senales de placeholder en produccion
- SEO tecnico incompleto
- propuesta de valor poco diferenciada
- blog sin autoridad editorial real
- poco uso de activos diferenciales como el panel de cliente, el stack Rust y el enfoque multilingue

---

## Estado actual

### Ya hecho
1. **Credibilidad**
   - Copyright corregido a `Nakomi Studio` en ES/EN/JA.
   - Testimonios falsos eliminados.
   - Bug de hooks en `SeccionTestimonios` corregido.
   - Marcas ficticias eliminadas; quedan solo clientes reales.
   - CTA principal del hero conectado a `/contacto/`.

2. **SEO**
   - `SEOHead` ya usa una imagen OG real existente.
   - Se anadieron datos de imagen y tags OG/Twitter mas completos.
   - `schemas.ts` enriquecido con `LocalBusiness` / `ProfessionalService`.
   - `NosotrosIsland` ahora inyecta `Person` schemas del equipo real.
   - Sitemap dinamico ampliado para incluir posts publicados del blog.
   - Meta description de la home reescrita con posicionamiento real.

3. **Posicionamiento**
   - Hero reescrito en ES/EN/JA con diferenciadores reales.
   - Badges anadidos bajo el hero: Rust, enfoque editorial, multilingue.

### Bloqueado
1. **Post test del blog**
   - Sigue requiriendo accion manual desde el admin.
   - Post ID: `633f9f5d-af40-448d-bf89-3b2322f27895`
   - Accion: cambiar a `draft`.

### Pendiente real
1. Meta descriptions unicas para servicios y proyectos.
2. Pagina `/proceso`.
3. Resaltar el panel de cliente como diferencial visible.
4. Case studies reales de proyectos.
5. Contenido editorial real para blog.
6. LinkedIn reales del equipo.
7. Evaluacion seria de prerender/SSR.

---

## FASE 1 — Credibilidad inmediata

### 1A. Corregir placeholders visibles
**Estado:** Hecho

Incluyo aqui lo ya ejecutado:
- copyright
- testimonios falsos
- marcas ficticias
- CTA roto

### 1B. Limpiar blog de contenido de prueba
**Estado:** Bloqueado manualmente

**Accion pendiente**
- entrar al admin
- despublicar el post `test`

**Por que importa**
- ahora mismo afecta confianza, SEO y percepcion de calidad

---

## FASE 2 — SEO tecnico

### 2A. Open Graph y Twitter Cards
**Estado:** Hecho

**Implementado**
- imagen por defecto real
- `og:image`, `og:locale`, `twitter:image`
- width/height/alt para OG

**Mejora futura recomendable**
- crear una imagen OG dedicada 1200x630 de marca en lugar de reutilizar una portada

### 2B. Schema.org
**Estado:** Hecho

**Implementado**
- `LocalBusiness` / `ProfessionalService`
- direccion de Copenhague
- `contactPoint`
- `Person` para miembros del equipo

**Mejora futura recomendable**
- anadir `sameAs` cuando existan perfiles sociales reales

### 2C. Meta descriptions para servicios y proyectos
**Estado:** Pendiente

**Objetivo**
- que cada servicio y proyecto tenga un snippet util y comercial en buscadores

**Superficies**
- proyectos: Kamples, Mabuhay, Task Manager
- servicios: 5 servicios principales

**Enfoque recomendado**
- escribir `meta_title` y `meta_description` manuales, no genericos
- resaltar el problema que resuelve cada servicio
- evitar copy repetido entre paginas

### 2D. Sitemap dinamico
**Estado:** Hecho

**Implementado**
- rutas estaticas
- servicios
- proyectos
- posts publicados

### 2E. Prerender / SSR
**Estado:** Pendiente estrategico

**Por que importa**
- el SEO de una SPA sigue limitado aunque mejores metas y sitemap

**Opciones**
1. prerender de paginas clave
2. SSR parcial desde Axum
3. snapshot estatico solo para marketing pages

**Recomendacion**
- tratarlo como iniciativa separada de arquitectura

---

## FASE 3 — Posicionamiento y conversion

### 3A. Reescritura del hero
**Estado:** Hecho

**Nuevo eje**
- estudio en Copenhague
- desarrollo en Rust
- trabajo multilingue
- criterio editorial

### 3B. Badges de diferenciacion
**Estado:** Hecho

**Objetivo**
- hacer visibles los diferenciales en el primer scroll

### 3C. Pagina `/proceso`
**Estado:** Pendiente

**Objetivo**
- explicar como trabaja Nakomi y reducir friccion antes del contacto

**Estructura recomendada**
1. Briefing
2. Propuesta
3. Diseno
4. Desarrollo
5. Lanzamiento
6. Soporte / evolucion

**CTA**
- llevar a contacto
- enlazar desde footer y desde servicios

### 3D. Panel de cliente como diferencial
**Estado:** Pendiente

**Objetivo**
- convertir una capacidad interna real en argumento comercial

**Recomendacion**
- crear una seccion visual en `/servicios` o `/nosotros`
- mostrar:
  - seguimiento de proyecto
  - entregables
  - comunicacion
  - pagos o fases si aplica

---

## FASE 4 — Contenido

### 4A. Case studies reales
**Estado:** Pendiente

**Regla**
- sin metricas

**Formato recomendado**
1. contexto
2. reto
3. decisiones creativas y tecnicas
4. solucion construida
5. stack o enfoque

**Prioridad**
- Kamples
- Mabuhay

### 4B. Blog real
**Estado:** Pendiente

**Minimo recomendado antes de empujarlo fuerte**
- 3 articulos solidos

**Temas iniciales**
1. por que usamos Rust en backend
2. como migramos de WordPress a Axum
3. que hace que una web creativa siga siendo rapida

### 4C. LinkedIn del equipo
**Estado:** Pendiente

**Objetivo**
- mejorar confianza y validacion humana del estudio

---

## Orden recomendado de ejecucion desde ahora

### Siguiente bloque util
1. **2C** meta descriptions de servicios y proyectos
2. **3C** pagina `/proceso`
3. **3D** seccion del panel de cliente

### Bloque posterior
1. **4A** case studies reales
2. **4B** 3 articulos de blog
3. **4C** LinkedIn del equipo

### Bloque estrategico separado
1. **2E** prerender/SSR

---

## Otras cosas que tambien se pueden hacer

### Prioridad alta
1. **FAQ SEO por servicio**
   - una FAQ breve por pagina de servicio
   - ayuda a resolver objeciones y ampliar semantica de busqueda

2. **Formularios con mejor pre-cualificacion**
   - anadir campos tipo:
     - tipo de proyecto
     - plazo estimado
     - idioma preferido
     - referencia visual
   - mejora calidad de lead sin tocar precios

3. **Prueba social real sin inventar nada**
   - quotes cortas de clientes reales
   - logos reales
   - screenshots reales de entregables o interfaces

4. **Landing pages por vertical**
   - restaurantes
   - ecommerce
   - estudios creativos
   - SaaS pequeno
   - hospitality

### Prioridad media
1. **Lead magnet editorial**
   - checklist para preparar una web nueva
   - guia de contenido para brief creativo

2. **Comparativa de enfoque**
   - explicar por que Nakomi no vende solo "diseno bonito"
   - posicionar combinacion de estetica + rendimiento + estructura

3. **Biblioteca visual de procesos**
   - mockups
   - wireframes
   - fragmentos reales de UI

4. **Mejor interlinking**
   - proyectos enlazando a servicios
   - servicios enlazando a proceso
   - blog enlazando a proyectos y contacto

### Prioridad estrategica
1. **Pages localizadas de verdad**
   - no solo traducidas
   - adaptar copy por idioma / mercado

2. **Sistema de captacion de testimonios reales**
   - flujo post-entrega
   - formato simple para pedir feedback

3. **Newsletter o journal por email**
   - solo si el equipo puede mantener ritmo editorial

4. **Medicion de conversion**
   - eventos de CTA
   - formularios enviados
   - clicks a contacto
   - sin depender de vanity metrics

---

## Lo que yo haria a continuacion

Si el objetivo es mejorar el sitio de forma visible y util sin dispersarse:

1. cerrar `2C`
2. construir `/proceso`
3. mostrar el panel de cliente
4. escribir 2 case studies reales
5. dejar el blog con 3 piezas fundacionales

Ese orden mejora antes:
- snippet en Google
- conversion
- claridad comercial
- confianza real
- autoridad editorial

---

## Notas
- El plan completo ya no asume cosas que en codigo ya estan resueltas.
- Lo bloqueado manualmente sigue explicitado para no perderlo.
- Si quieres, el siguiente bloque natural es ejecutar **2C + 3C + 3D**.
