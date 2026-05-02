/* [064A-62] Sección admin: configuración y herramientas de desarrollo.
 * Botones para recrear/borrar datos de prueba (seed).
 * Solo visible para admin.
 * [084A-11] Lógica extraída a useSeccionConfiguracion (max 3 useState).
 * [114A-12] Toggle de rotación de API keys con status. */

import { Loader2, AlertCircle, CheckCircle2, Database, Trash2, RotateCw, Key, RefreshCw } from 'lucide-react';
import { useSeccionConfiguracion } from '../../hooks/useSeccionConfiguracion';
import { useRotacionApi } from '../../hooks/useRotacionApi';
import { useFixtureSync } from '../../hooks/useFixtureSync';
import { Button } from '../ui/Button';
import { Modal } from '../ui/Modal';
import './SeccionConfiguracion.css';

export function SeccionConfiguracion() {
    const { cargando, resultado, errorMsg, confirmando, setConfirmando, ejecutarSeed } = useSeccionConfiguracion();
    const { status: rotacion, cargando: rotacionCargando, errorMsg: rotacionError, toggle } = useRotacionApi();
    const { syncResult, statusData, cargando: syncCargando, error: syncError, sync } = useFixtureSync();

    return (
        <div className="configSeccion">
            {/* [114A-12] Bloque de rotación de API keys */}
            <div className="configBloque">
                <h2 className="configBloqueTitle">
                    <Key size={18} />
                    Rotación de API Keys
                </h2>
                <p className="configBloqueDesc">
                    Controla la rotación automática de claves API para el chatbot.
                    Cuando está activa, cada mensaje usa una clave diferente (round-robin).
                    Desactivada, siempre usa la primera clave.
                </p>

                {rotacion && (
                    <div className="configRotacionStatus">
                        <div className="configRotacionIndicador">
                            <span className={`configRotacionDot ${rotacion.enabled ? 'configRotacionDotActivo' : 'configRotacionDotInactivo'}`} />
                            <span className="configRotacionLabel">
                                {rotacion.enabled ? 'Rotación activa' : 'Rotación desactivada'}
                            </span>
                        </div>
                        <div className="configRotacionDetalles">
                            <span>Keys configuradas: <strong>{rotacion.total_keys}</strong></span>
                            <span>Key actual: <strong>#{rotacion.current_index + 1}</strong></span>
                            <span>Modelo: <strong>{rotacion.model}</strong></span>
                            {rotacion.has_fallback && <span className="configRotacionFallback">Fallback: Gemini</span>}
                        </div>
                    </div>
                )}

                <div className="configAcciones">
                    <Button
                        onClick={() => toggle(!rotacion?.enabled)}
                        disabled={rotacionCargando || !rotacion}
                    >
                        {rotacionCargando ? <Loader2 size={16} className="configSpinner" /> : <RotateCw size={16} />}
                        {rotacion?.enabled ? 'Desactivar Rotación' : 'Activar Rotación'}
                    </Button>
                </div>

                {rotacionError && (
                    <div className="configMensaje configMensajeError">
                        <AlertCircle size={16} />
                        <span>{rotacionError}</span>
                    </div>
                )}
            </div>

            <div className="configBloque">
                <h2 className="configBloqueTitle">Datos de Prueba</h2>
                <p className="configBloqueDesc">
                    Gestiona los datos de prueba del sistema. Recrear crea usuarios y órdenes
                    de ejemplo para testear flujos. Borrar elimina únicamente los datos generados por seed.
                </p>

                <div className="configAcciones">
                    <Button
                        onClick={() => setConfirmando('recrear')}
                        disabled={cargando}
                    >
                        {cargando ? <Loader2 size={16} className="configSpinner" /> : <Database size={16} />}
                        Recrear Datos de Prueba
                    </Button>

                    <Button
                        variante="secundario"
                        onClick={() => setConfirmando('borrar')}
                        disabled={cargando}
                    >
                        {cargando ? <Loader2 size={16} className="configSpinner" /> : <Trash2 size={16} />}
                        Borrar Datos de Prueba
                    </Button>
                </div>

                {resultado && (
                    <div className="configMensaje configMensajeExito">
                        <CheckCircle2 size={16} />
                        <span>{resultado}</span>
                    </div>
                )}

                {errorMsg && (
                    <div className="configMensaje configMensajeError">
                        <AlertCircle size={16} />
                        <span>{errorMsg}</span>
                    </div>
                )}
            </div>

            <Modal
                abierto={confirmando !== null}
                onCerrar={() => setConfirmando(null)}
            >
                <div className="configConfirmContenido">
                    <p className="configConfirmTexto">
                        {confirmando === 'recrear'
                            ? '¿Recrear datos de prueba? Se borrarán los existentes y se crearán nuevos usuarios y órdenes de test.'
                            : '¿Borrar todos los datos de prueba? Esta acción elimina usuarios test y sus órdenes/fases/chat.'}
                    </p>
                    <div className="modalAcciones">
                        <Button variante="secundario" tamano="pequeno" onClick={() => setConfirmando(null)}>
                            Cancelar
                        </Button>
                        <Button tamano="pequeno" onClick={() => ejecutarSeed(confirmando)}>
                            Confirmar
                        </Button>
                    </div>
                </div>
            </Modal>

            {/* [154A-2] Bloque de sincronización de fixtures de contenido */}
            <div className="configBloque">
                <h2 className="configBloqueTitle">
                    <RefreshCw size={18} />
                    Sincronizar Contenido
                </h2>
                <p className="configBloqueDesc">
                    Aplica los archivos TOML de <code>content/</code> a la base de datos sin reiniciar el servidor.
                    Útil cuando se editan fixtures de servicios, proyectos, equipo u otro contenido directamente.
                </p>

                {statusData && (
                    <div className="configRotacionDetalles">
                        <span>Registros rastreados: <strong>{statusData.tracked_records}</strong></span>
                        {statusData.tables.map((t: { table_name: string; record_count: number }) => (
                            <span key={t.table_name}>{t.table_name}: <strong>{t.record_count}</strong></span>
                        ))}
                    </div>
                )}

                <div className="configAcciones">
                    <Button onClick={sync} disabled={syncCargando}>
                        {syncCargando ? <Loader2 size={16} className="configSpinner" /> : <RefreshCw size={16} />}
                        Sincronizar Fixtures
                    </Button>
                </div>

                {syncResult && (
                    <div className="configMensaje configMensajeExito">
                        <CheckCircle2 size={16} />
                        <span>
                            Sincronizado: +{syncResult.inserted} insertados, ~{syncResult.updated} actualizados,
                            -{syncResult.deleted} borrados, {syncResult.skipped} sin cambios
                            {syncResult.errors.length > 0 && ` — ${syncResult.errors.length} error(es)`}
                        </span>
                    </div>
                )}

                {syncError && (
                    <div className="configMensaje configMensajeError">
                        <AlertCircle size={16} />
                        <span>{syncError}</span>
                    </div>
                )}
            </div>
        </div>
    );
}
