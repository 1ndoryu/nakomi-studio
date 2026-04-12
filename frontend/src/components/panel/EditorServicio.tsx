/* [074A-9] Editor de servicio CMS — Modal con tabs: General, Media, Planes, SEO.
 * Reutiliza componentes existentes: Modal, Input, SlugInput, UploadImage, RichTextEditor, Textarea.
 * Lógica de formulario extraída a useEditorServicio para cumplir max 3 useState.
 * [074A-66] Tab Planes agregada para editar planes de pricing inline.
 * sentinel-disable-file html-nativo-en-vez-de-componente: Tabs del editor y status toggles usan
 * <button> nativo porque botonBase interfiere con estilos del tab (mismo patrón 074A-47). */
import React, {useState, useCallback} from 'react';
import {Modal} from '../ui/Modal';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';
import {Button} from '../ui/Button';
import {SlugInput} from '../ui/SlugInput';
import {UploadImage} from '../ui/UploadImage';
import {RichTextEditor} from '../ui/RichTextEditor';
import {TabPlanes} from './TabPlanes';
import {useEditorServicio} from '../../hooks/useEditorServicio';
import type {AdminService, CreateServiceBody, UpdateServiceBody, SavePlanBody} from '../../api/admin-services';
import './EditorServicio.css';

type TabEditor = 'general' | 'media' | 'planes' | 'seo';

interface EditorServicioProps {
    abierto: boolean;
    onCerrar: () => void;
    servicio: AdminService | null;
    onGuardar: (body: CreateServiceBody | UpdateServiceBody, planes: SavePlanBody[]) => Promise<void>;
    guardando: boolean;
}

const TABS: {id: TabEditor; label: string}[] = [
    {id: 'general', label: 'General'},
    {id: 'media', label: 'Media'},
    {id: 'planes', label: 'Planes'},
    {id: 'seo', label: 'SEO'},
];

export const EditorServicio: React.FC<EditorServicioProps> = ({
    abierto,
    onCerrar,
    servicio,
    onGuardar,
    guardando,
}) => {
    const [tab, setTab] = useState<TabEditor>('general');
    const form = useEditorServicio(servicio, abierto);

    const handleGuardar = useCallback(async () => {
        await onGuardar(form.buildBody(), form.buildPlansBody());
    }, [form, onGuardar]);

    return (
        <Modal abierto={abierto} onCerrar={onCerrar} className="editorServicioModal modalSinPadding">
            <div className="editorServicioTabs">
                {TABS.map(t => (
                    <button
                        key={t.id}
                        type="button"
                        className={`editorServicioTab ${tab === t.id ? 'editorServicioTab--activo' : ''}`}
                        onClick={() => setTab(t.id)}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            <div className="editorServicioCuerpo">
                {tab === 'general' && (
                    <div className="editorServicioSeccion">
                        <label className="editorServicioLabel">
                            Título
                            <Input
                                value={form.titulo}
                                onChange={e => form.setTitulo(e.target.value)}
                                placeholder="Nombre del servicio"
                            />
                        </label>

                        <SlugInput titulo={form.titulo} valor={form.slug} onChange={form.setSlug} />

                        <label className="editorServicioLabel">
                            Descripción breve
                            <Textarea
                                className="editorServicioTextarea"
                                value={form.descripcion}
                                onChange={e => form.setDescripcion(e.target.value)}
                                placeholder="Descripción corta del servicio"
                                rows={3}
                            />
                        </label>

                        <label className="editorServicioLabel">
                            Contenido
                        </label>
                        <RichTextEditor
                            contenido={form.contenido}
                            onChange={form.setContenido}
                            placeholder="Descripción detallada del servicio..."
                        />

                        <div className="editorServicioFila">
                            <label className="editorServicioLabel editorServicioLabelCorto">
                                Precio base (cents)
                                <Input
                                    type="number"
                                    value={form.precioCents}
                                    onChange={e => form.setPrecioCents(Number(e.target.value))}
                                    min={0}
                                />
                            </label>

                        </div>
                    </div>
                )}

                {tab === 'media' && (
                    <div className="editorServicioSeccion">
                        <UploadImage
                            valor={form.imagenUrl}
                            onChange={form.setImagenUrl}
                            etiqueta="Imagen principal"
                        />
                    </div>
                )}

                {tab === 'planes' && (
                    <div className="editorServicioSeccion">
                        <TabPlanes planes={form.planes} onChange={form.setPlanes} />
                    </div>
                )}

                {tab === 'seo' && (
                    <div className="editorServicioSeccion">
                        <label className="editorServicioLabel">
                            Meta título
                            <Input
                                value={form.metaTitle}
                                onChange={e => form.setMetaTitle(e.target.value)}
                                placeholder="Título para buscadores"
                                maxLength={70}
                            />
                        </label>

                        <label className="editorServicioLabel">
                            Meta descripción
                            <Textarea
                                className="editorServicioTextarea"
                                value={form.metaDescription}
                                onChange={e => form.setMetaDescription(e.target.value)}
                                placeholder="Descripción para buscadores (max 160 caracteres)"
                                rows={3}
                                maxLength={160}
                            />
                        </label>

                        <div className="editorServicioSeoPreview">
                            <span className="editorServicioSeoPreviewLabel">Vista previa en Google</span>
                            <div className="editorServicioSeoCard">
                                <span className="editorServicioSeoTitulo">
                                    {form.metaTitle || form.titulo || 'Título del servicio'}
                                </span>
                                <span className="editorServicioSeoUrl">
                                    nakomi.studio/servicios/{form.slug || 'slug'}
                                </span>
                                <span className="editorServicioSeoDesc">
                                    {form.metaDescription || form.descripcion || 'Descripción del servicio...'}
                                </span>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            <div className="editorServicioAcciones">
                <div className="editorServicioEstado">
                    <button
                        type="button"
                        className={`editorServicioStatusBtn ${form.status === 'draft' ? 'editorServicioStatusBtn--activo' : ''}`}
                        onClick={() => form.setStatus('draft')}
                    >
                        Borrador
                    </button>
                    <button
                        type="button"
                        className={`editorServicioStatusBtn ${form.status === 'published' ? 'editorServicioStatusBtn--activo' : ''}`}
                        onClick={() => form.setStatus('published')}
                    >
                        Publicado
                    </button>
                </div>
                <Button
                    variante="primario"
                    tamano="pequeno"
                    onClick={handleGuardar}
                    disabled={guardando || !form.titulo.trim() || !form.slug.trim()}
                >
                    {guardando ? 'Guardando...' : 'Guardar'}
                </Button>
            </div>
        </Modal>
    );
};
