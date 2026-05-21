/* [044A-43] Componente: SeccionPerfil
 * Formulario de configuración de perfil. Conectado al backend via usePerfil.
 * El botón "Cambiar foto" abre un input file oculto que sube el avatar al servidor.
 * [205A-2] Añade cambio de contraseña y deja explícito que el cambio de correo no verifica por email.
 * [205A-3] Si cambia el email, abre un modal que obliga a reescribirlo antes de guardar. */
import React, {useRef} from 'react';
import {useTranslation} from 'react-i18next';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Modal, ModalBody, ModalField, ModalLabel} from '../ui/Modal';
import OptimizedImage from '../ui/OptimizedImage';
import {Textarea} from '../ui/Textarea';
import {useSeccionPerfil} from '../../hooks/useSeccionPerfil';
import './SeccionPerfil.css';

export const SeccionPerfil: React.FC = () => {
    const {t} = useTranslation();
    const {
        estado, guardado, guardando, errorGuardar, cargando, avatarUrl,
        subiendoAvatar, estadoPassword, passwordActualizada, guardandoPassword,
        errorPassword, actualizarCampo, actualizarPasswordCampo,
        handleGuardarPassword, modalCambioEmailAbierto,
        confirmacionEmail, errorConfirmacionEmail, emailObjetivo,
        cerrarModalCambioEmail, setConfirmacionEmail,
        limpiarErrorConfirmacionEmail, handleSubmitPerfil,
        confirmarCambioEmail, alSeleccionarArchivo,
    } = useSeccionPerfil();
    const inputArchivoRef = useRef<HTMLInputElement>(null);

    if (cargando) {
        return <div className="perfilSeccion"><p>{t('common.loading', 'Cargando...')}</p></div>;
    }

    return (
        <div className="perfilSeccion">
            {/* sentinel-disable-next-line modal-estructura-no-canonica */}
            <form className="perfilFormulario" onSubmit={handleSubmitPerfil}>
                {/* Avatar */}
                <div className="perfilAvatarSeccion">
                    <div className="perfilAvatar">
                        <OptimizedImage
                            src={avatarUrl}
                            alt={t('accessibility.profile_photo')}
                            loading="eager"
                        />
                    </div>
                    <div className="perfilAvatarAcciones">
                        <input
                            ref={inputArchivoRef}
                            type="file"
                            accept="image/jpeg,image/png,image/webp,image/gif"
                            onChange={alSeleccionarArchivo}
                            className="perfilArchivoOculto"
                        />
                        <Button
                            variante="outline"
                            tamano="pequeno"
                            type="button"
                            onClick={() => inputArchivoRef.current?.click()}
                            disabled={subiendoAvatar}
                        >
                            {subiendoAvatar ? t('common.uploading', 'Subiendo...') : t('panel.change_photo')}
                        </Button>
                        <span className="perfilAvatarNota">{t('panel.photo_help')}</span>
                    </div>
                </div>

                <div className="perfilFormGrid">
                    <div className="perfilCampo">
                        <label htmlFor="perfilNombre" className="perfilCampoEtiqueta">{t('panel.display_name')}</label>
                        <Input
                            type="text"
                            id="perfilNombre"
                            value={estado.nombre}
                            onChange={(e) => actualizarCampo('nombre', e.target.value)}
                            placeholder={t('panel.display_name_placeholder')}
                            className="perfilCampoInput"
                        />
                    </div>
                    <div className="perfilCampo">
                        <label htmlFor="perfilEmail" className="perfilCampoEtiqueta">{t('panel.email')}</label>
                        <Input
                            type="email"
                            id="perfilEmail"
                            value={estado.email}
                            onChange={(e) => actualizarCampo('email', e.target.value)}
                            className="perfilCampoInput"
                        />
                        <span className="perfilAvatarNota">Si lo cambias, se te pedirá confirmarlo escribiéndolo de nuevo antes de guardar.</span>
                    </div>
                </div>

                <div className="perfilCampo">
                    <label htmlFor="perfilDescripcion" className="perfilCampoEtiqueta">{t('panel.bio_label')}</label>
                    <Textarea
                        id="perfilDescripcion"
                        value={estado.descripcion}
                        onChange={(e) => actualizarCampo('descripcion', e.target.value)}
                        placeholder={t('panel.bio_placeholder')}
                        className="perfilCampoTextarea"
                        rows={4}
                    />
                </div>

                <h3 className="perfilSubseccionTitulo">{t('panel.social_title')}</h3>
                <div className="perfilFormGrid">
                    <div className="perfilCampo">
                        <label htmlFor="perfilLinkedin" className="perfilCampoEtiqueta">LinkedIn</label>
                        <Input
                            type="url"
                            id="perfilLinkedin"
                            value={estado.linkedin}
                            onChange={(e) => actualizarCampo('linkedin', e.target.value)}
                            placeholder="https://linkedin.com/in/tu-perfil"
                            className="perfilCampoInput"
                        />
                    </div>
                    <div className="perfilCampo">
                        <label htmlFor="perfilTwitter" className="perfilCampoEtiqueta">Twitter / X</label>
                        <Input
                            type="url"
                            id="perfilTwitter"
                            value={estado.twitter}
                            onChange={(e) => actualizarCampo('twitter', e.target.value)}
                            placeholder="https://x.com/tu-usuario"
                            className="perfilCampoInput"
                        />
                    </div>
                    <div className="perfilCampo">
                        <label htmlFor="perfilWebsite" className="perfilCampoEtiqueta">{t('panel.website')}</label>
                        <Input
                            type="url"
                            id="perfilWebsite"
                            value={estado.website}
                            onChange={(e) => actualizarCampo('website', e.target.value)}
                            placeholder="https://tu-sitio.com"
                            className="perfilCampoInput"
                        />
                    </div>
                </div>

                <div className="perfilFormAcciones">
                    <Button variante="primario" tamano="mediano" disabled={guardando}>
                        {guardando ? t('common.saving', 'Guardando...') : t('panel.save')}
                    </Button>
                    {guardado && <span className="perfilGuardadoExito">{t('panel.saved_success')}</span>}
                    {errorGuardar && <span className="perfilGuardadoError">{errorGuardar}</span>}
                </div>
            </form>

            <section className="perfilSeguridadBloque">
                <h3 className="perfilSubseccionTitulo">{t('panel.security_title', 'Seguridad')}</h3>
                <p className="perfilSeguridadDescripcion">
                    {t('panel.security_password_help', 'Cambia tu contraseña de acceso. Cuando se procese, el sistema enviará un correo informativo al email actual de la cuenta.')}
                </p>
                {/* sentinel-disable-next-line modal-estructura-no-canonica */}
                <form className="perfilFormulario" onSubmit={handleGuardarPassword}>
                    <div className="perfilFormGrid">
                        <div className="perfilCampo">
                            <label htmlFor="perfilPasswordActual" className="perfilCampoEtiqueta">{t('panel.current_password', 'Contraseña actual')}</label>
                            <Input
                                type="password"
                                id="perfilPasswordActual"
                                value={estadoPassword.actual}
                                onChange={(e) => actualizarPasswordCampo('actual', e.target.value)}
                                className="perfilCampoInput"
                                autoComplete="current-password"
                            />
                        </div>
                        <div className="perfilCampo">
                            <label htmlFor="perfilPasswordNueva" className="perfilCampoEtiqueta">{t('panel.new_password', 'Nueva contraseña')}</label>
                            <Input
                                type="password"
                                id="perfilPasswordNueva"
                                value={estadoPassword.nueva}
                                onChange={(e) => actualizarPasswordCampo('nueva', e.target.value)}
                                className="perfilCampoInput"
                                autoComplete="new-password"
                                minLength={8}
                            />
                        </div>
                    </div>

                    <div className="perfilCampo">
                        <label htmlFor="perfilPasswordConfirmar" className="perfilCampoEtiqueta">{t('panel.confirm_new_password', 'Confirmar nueva contraseña')}</label>
                        <Input
                            type="password"
                            id="perfilPasswordConfirmar"
                            value={estadoPassword.confirmar}
                            onChange={(e) => actualizarPasswordCampo('confirmar', e.target.value)}
                            className="perfilCampoInput"
                            autoComplete="new-password"
                            minLength={8}
                        />
                    </div>

                    <div className="perfilFormAcciones">
                        <Button variante="primario" tamano="mediano" type="submit" disabled={guardandoPassword}>
                            {guardandoPassword ? t('common.saving', 'Guardando...') : t('panel.update_password', 'Actualizar contraseña')}
                        </Button>
                        {passwordActualizada && <span className="perfilGuardadoExito">{t('panel.password_updated', 'Contraseña actualizada correctamente')}</span>}
                        {errorPassword && <span className="perfilGuardadoError">{errorPassword}</span>}
                    </div>
                </form>
            </section>

            <Modal abierto={modalCambioEmailAbierto} onCerrar={cerrarModalCambioEmail} className="modalCompacto">
                <ModalBody as="form" onSubmit={confirmarCambioEmail} className="perfilModalConfirmacion">
                    <div className="perfilModalResumen">
                        <p className="modalTexto">
                            {t('panel.confirm_email_change_text', 'Por seguridad, escribe otra vez el nuevo correo exactamente como quieres guardarlo antes de aplicar el cambio.')}
                        </p>
                        <p className="perfilModalEmailObjetivo">{emailObjetivo}</p>
                    </div>

                    <ModalField>
                        <ModalLabel htmlFor="perfilConfirmacionCambioEmail">
                            {t('panel.confirm_email_change_label', 'Escribe de nuevo el nuevo correo')}
                        </ModalLabel>
                        <Input
                            type="email"
                            id="perfilConfirmacionCambioEmail"
                            value={confirmacionEmail}
                            onChange={(e) => {
                                setConfirmacionEmail(e.target.value);
                                if (errorConfirmacionEmail) {
                                    limpiarErrorConfirmacionEmail();
                                }
                            }}
                            className="modalInput"
                            autoComplete="off"
                        />
                    </ModalField>

                    {errorConfirmacionEmail && <p className="perfilModalError">{errorConfirmacionEmail}</p>}

                    <div className="modalAcciones">
                        <Button
                            variante="outline"
                            tamano="pequeno"
                            type="button"
                            onClick={cerrarModalCambioEmail}
                            disabled={guardando}
                        >
                            {t('common.cancel', 'Cancelar')}
                        </Button>
                        <Button variante="primario" tamano="pequeno" type="submit" disabled={guardando}>
                            {guardando ? t('common.saving', 'Guardando...') : t('panel.confirm_email_change_action', 'Confirmar y guardar')}
                        </Button>
                    </div>
                </ModalBody>
            </Modal>
        </div>
    );
};
