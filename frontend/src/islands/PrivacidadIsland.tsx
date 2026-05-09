/* [095A-5] Página de Política de Privacidad.
 * Accesible desde el footer → /politica-privacidad.
 * Contenido legal estático; actualizar fechas y secciones según evolucione el servicio. */
import {LayoutPagina} from '../components/layout/LayoutPagina';
import {SEOHead} from '../components/seo/SEOHead';
import './PrivacidadIsland.css';

export const PrivacidadIsland = (): JSX.Element => {
    return (
        <LayoutPagina className="privacidadMain" id="paginaPrivacidad">
            <SEOHead
                title="Política de Privacidad"
                description="Política de privacidad de Nakomi Studio. Cómo recopilamos, usamos y protegemos tu información."
                path="/politica-privacidad"
            />

            <section className="privacidadContenido">
                <div className="privacidadContenedor">
                    <h1 className="privacidadTitulo">Política de Privacidad</h1>
                    <p className="privacidadFecha">Última actualización: mayo 2026</p>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">1. Responsable del tratamiento</h2>
                        <p>
                            Nakomi Studio es el responsable del tratamiento de los datos personales
                            recabados a través de este sitio web y los servicios asociados.
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">2. Datos que recopilamos</h2>
                        <p>Podemos recopilar los siguientes datos:</p>
                        <ul>
                            <li>Nombre y apellidos</li>
                            <li>Dirección de correo electrónico</li>
                            <li>Número de teléfono (opcional)</li>
                            <li>Información del proyecto o consulta</li>
                            <li>Datos técnicos de navegación (cookies, IP anonimizada)</li>
                        </ul>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">3. Finalidad del tratamiento</h2>
                        <p>Utilizamos tus datos para:</p>
                        <ul>
                            <li>Gestionar tu solicitud de contacto o presupuesto</li>
                            <li>Prestar los servicios contratados</li>
                            <li>Enviarte comunicaciones sobre el estado de tu proyecto</li>
                            <li>Mejorar nuestros servicios (datos anonimizados)</li>
                            <li>Enviarte nuestra newsletter, si te has suscrito voluntariamente</li>
                        </ul>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">4. Base legal</h2>
                        <p>
                            El tratamiento se basa en el consentimiento del interesado (Art. 6.1.a RGPD)
                            y en la ejecución del contrato o medidas precontractuales (Art. 6.1.b RGPD).
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">5. Conservación de datos</h2>
                        <p>
                            Los datos se conservan durante el tiempo necesario para cumplir la finalidad
                            para la que fueron recabados y para atender las obligaciones legales aplicables.
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">6. Cesión a terceros</h2>
                        <p>
                            No cedemos datos personales a terceros salvo obligación legal o cuando sea
                            estrictamente necesario para la prestación del servicio (e.g. pasarela de pago,
                            proveedor de hosting). En estos casos, los terceros actúan como encargados del
                            tratamiento bajo acuerdos de confidencialidad.
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">7. Tus derechos</h2>
                        <p>Puedes ejercer en cualquier momento los siguientes derechos:</p>
                        <ul>
                            <li><strong>Acceso</strong> a tus datos personales</li>
                            <li><strong>Rectificación</strong> de datos inexactos</li>
                            <li><strong>Supresión</strong> («derecho al olvido»)</li>
                            <li><strong>Limitación</strong> del tratamiento</li>
                            <li><strong>Portabilidad</strong> de los datos</li>
                            <li><strong>Oposición</strong> al tratamiento</li>
                        </ul>
                        <p>
                            Para ejercerlos, contáctanos a través del chat de esta web o escríbenos
                            directamente. Responderemos en el plazo legal de 30 días.
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">8. Cookies</h2>
                        <p>
                            Este sitio puede utilizar cookies técnicas necesarias para el funcionamiento
                            básico. No utilizamos cookies de seguimiento ni publicidad de terceros sin
                            tu consentimiento explícito.
                        </p>
                    </div>

                    <div className="privacidadSeccion">
                        <h2 className="privacidadSubtitulo">9. Cambios en esta política</h2>
                        <p>
                            Nos reservamos el derecho a actualizar esta política para reflejar cambios
                            en nuestras prácticas o en la normativa aplicable. La fecha de «última
                            actualización» indica cuándo se realizaron los últimos cambios.
                        </p>
                    </div>
                </div>
            </section>
        </LayoutPagina>
    );
};
