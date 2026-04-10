/**
 * Componente: ModalAutenticacion
 * Modal con tabs para Iniciar Sesion / Registrarse.
 * Incluye: login, registro, recuperacion de contrasena, login con Google.
 * La logica de estado se delega al hook useAutenticacion (SRP).
 * [044A-13] Conectado con backend REST API. Login y registro funcionales.
 * Pendiente: OAuth Google, reCAPTCHA, recuperación de contraseña real.
 */
import React from 'react';
import {useTranslation} from 'react-i18next';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Modal} from '../ui/Modal';
import {useAutenticacion} from '../../hooks/useAutenticacion';
import './ModalAutenticacion.css';

interface ModalAutenticacionProps {
    abierto: boolean;
    onCerrar: () => void;
}

export const ModalAutenticacion: React.FC<ModalAutenticacionProps> = ({abierto, onCerrar}) => {
    const {t} = useTranslation();
    const {
        vista, setVista, cargando, error,
        login, registro, recuperar,
        actualizarLogin, actualizarRegistro, actualizarRecuperar,
        handleLogin, handleRegistro, handleRecuperar,
        handleGoogleLogin, resetRecuperacion
    } = useAutenticacion(onCerrar);

    return (
        <Modal abierto={abierto} onCerrar={onCerrar} className="modalAutenticacion">
                {/* [044A-15] Eliminado boton cerrar y titulo por solicitud del usuario.
                 * Se cierra haciendo click fuera del modal (overlay) o con Escape. */}

                {/* Tabs (solo login/registro) */}
                {vista !== 'recuperar' && (
                    <div className="modalTabs">
                        <Button
                            variante="texto"
                            className={`modalTab ${vista === 'login' ? 'modalTabActivo' : ''}`}
                            onClick={() => setVista('login')}
                            type="button"
                        >
                            {t('auth.login')}
                        </Button>
                        <Button
                            variante="texto"
                            className={`modalTab ${vista === 'registro' ? 'modalTabActivo' : ''}`}
                            onClick={() => setVista('registro')}
                            type="button"
                        >
                            {t('auth.register')}
                        </Button>
                    </div>
                )}

                {/* [044A-13] Mensaje de error visible al usuario */}
                {error && <p className="modalError" role="alert">{error}</p>}

                {/* Contenido: Login */}
                {vista === 'login' && (
                    <form className="modalFormulario" onSubmit={handleLogin}>
                        <div className="modalCampo">
                            <label htmlFor="loginEmail" className="modalEtiqueta">{t('auth.email_label')}</label>
                            <Input
                                type="email"
                                id="loginEmail"
                                value={login.email}
                                onChange={e => actualizarLogin('email', e.target.value)}
                                placeholder={t('auth.email_placeholder')}
                                className="modalInput"
                                required
                            />
                        </div>
                        <div className="modalCampo">
                            <label htmlFor="loginPassword" className="modalEtiqueta">{t('auth.password_label')}</label>
                            <Input
                                type="password"
                                id="loginPassword"
                                value={login.password}
                                onChange={e => actualizarLogin('password', e.target.value)}
                                placeholder={t('auth.password_placeholder')}
                                className="modalInput"
                                required
                            />
                        </div>
                        <Button
                            variante="texto"
                            type="button"
                            className="modalEnlaceTexto"
                            onClick={() => setVista('recuperar')}
                        >
                            {t('auth.forgot_password')}
                        </Button>
                        <Button variante="primario" tamano="mediano" className="modalAccionPrincipal" disabled={cargando}>
                            {cargando ? t('auth.loading') : t('auth.login')}
                        </Button>

                        <div className="modalSeparador">
                            <span>{t('auth.or')}</span>
                        </div>

                        <Button variante="texto" type="button" className="modalAccionGoogle" onClick={handleGoogleLogin}>
                            <svg width="18" height="18" viewBox="0 0 24 24">
                                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                            </svg>
                            {t('auth.google_login')}
                        </Button>
                    </form>
                )}

                {/* Contenido: Registro */}
                {vista === 'registro' && (
                    <form className="modalFormulario" onSubmit={handleRegistro}>
                        <div className="modalCampo">
                            <label htmlFor="regNombre" className="modalEtiqueta">{t('auth.name_label')}</label>
                            <Input
                                type="text"
                                id="regNombre"
                                value={registro.nombre}
                                onChange={e => actualizarRegistro('nombre', e.target.value)}
                                placeholder={t('auth.name_placeholder')}
                                className="modalInput"
                                required
                            />
                        </div>
                        <div className="modalCampo">
                            <label htmlFor="regEmail" className="modalEtiqueta">{t('auth.email_label')}</label>
                            <Input
                                type="email"
                                id="regEmail"
                                value={registro.email}
                                onChange={e => actualizarRegistro('email', e.target.value)}
                                placeholder={t('auth.email_placeholder')}
                                className="modalInput"
                                required
                            />
                        </div>
                        <div className="modalCampo">
                            <label htmlFor="regPassword" className="modalEtiqueta">{t('auth.password_label')}</label>
                            <Input
                                type="password"
                                id="regPassword"
                                value={registro.password}
                                onChange={e => actualizarRegistro('password', e.target.value)}
                                placeholder={t('auth.min_chars')}
                                className="modalInput"
                                minLength={8}
                                required
                            />
                        </div>
                        <div className="modalCampo">
                            <label htmlFor="regConfirmar" className="modalEtiqueta">{t('auth.confirm_password')}</label>
                            <Input
                                type="password"
                                id="regConfirmar"
                                value={registro.confirmar}
                                onChange={e => actualizarRegistro('confirmar', e.target.value)}
                                placeholder={t('auth.confirm_placeholder')}
                                className="modalInput"
                                minLength={8}
                                required
                            />
                        </div>
                        {/* TO-DO: Agregar reCAPTCHA aqui cuando se configuren las keys */}
                        <p className="modalNotaCaptcha">{t('auth.captcha_note')}</p>
                        <Button variante="primario" tamano="mediano" className="modalAccionPrincipal" disabled={cargando}>
                            {cargando ? t('auth.creating') : t('auth.register')}
                        </Button>

                        <div className="modalSeparador">
                            <span>{t('auth.or')}</span>
                        </div>

                        <Button variante="texto" type="button" className="modalAccionGoogle" onClick={handleGoogleLogin}>
                            <svg width="18" height="18" viewBox="0 0 24 24">
                                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                            </svg>
                            {t('auth.google_register')}
                        </Button>
                    </form>
                )}

                {/* Contenido: Recuperar contraseña */}
                {vista === 'recuperar' && (
                    <div className="modalFormulario">
                        {recuperar.enviado ? (
                            <div className="modalExito">
                                <p className="modalExitoTexto">{t('auth.recover_sent')}</p>
                                <Button variante="outline" onClick={resetRecuperacion}>
                                    {t('auth.back_login')}
                                </Button>
                            </div>
                        ) : (
                            <form onSubmit={handleRecuperar}>
                                <p className="modalDescripcion">{t('auth.recover_desc')}</p>
                                <div className="modalCampo">
                                    <label htmlFor="recEmail" className="modalEtiqueta">{t('auth.email_label')}</label>
                                    <Input
                                        type="email"
                                        id="recEmail"
                                        value={recuperar.email}
                                        onChange={e => actualizarRecuperar('email', e.target.value)}
                                        placeholder={t('auth.email_placeholder')}
                                        className="modalInput"
                                        required
                                    />
                                </div>
                                <Button variante="primario" tamano="mediano" className="modalAccionPrincipal" disabled={cargando}>
                                    {cargando ? t('auth.sending') : t('auth.send_link')}
                                </Button>
                                <Button variante="texto" type="button" className="modalEnlaceTexto" onClick={() => setVista('login')}>
                                    {t('auth.back_login_text')}
                                </Button>
                            </form>
                        )}
                    </div>
                )}
        </Modal>
    );
};
