/* [044A-43] Componente: SeccionPerfil
 * Formulario de configuración de perfil. Conectado al backend via usePerfil.
 * El botón "Cambiar foto" abre un input file oculto que sube el avatar al servidor. */
import React, {useRef} from 'react';
import {useTranslation} from 'react-i18next';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import OptimizedImage from '../ui/OptimizedImage';
import {Textarea} from '../ui/Textarea';
import {usePerfil} from '../../hooks/usePerfil';
import './SeccionPerfil.css';

export const SeccionPerfil: React.FC = () => {
    const {t} = useTranslation();
    const {
        estado, guardado, guardando, errorGuardar, cargando, perfil, avatarUrl,
        subiendoAvatar, actualizarCampo, handleGuardar, handleSubirAvatar
    } = usePerfil();
    const inputArchivoRef = useRef<HTMLInputElement>(null);

    const abrirSelectorArchivo = () => {
        inputArchivoRef.current?.click();
    };

    const alSeleccionarArchivo = (e: React.ChangeEvent<HTMLInputElement>) => {
        const archivo = e.target.files?.[0];
        if (archivo) {
            handleSubirAvatar(archivo);
            /* Limpiar para permitir subir el mismo archivo de nuevo */
            e.target.value = '';
        }
    };

    if (cargando) {
        return <div className="perfilSeccion"><p>{t('common.loading', 'Cargando...')}</p></div>;
    }

    return (
        <div className="perfilSeccion">
            {/* sentinel-disable-next-line modal-estructura-no-canonica */}
            <form className="perfilFormulario" onSubmit={handleGuardar}>
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
                            onClick={abrirSelectorArchivo}
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
                            value={perfil?.email || ''}
                            className="perfilCampoInput"
                            disabled
                        />
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
        </div>
    );
};
