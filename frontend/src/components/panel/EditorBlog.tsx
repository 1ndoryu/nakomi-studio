/* [batch-4] Status toggles migrados a <Button>. Las tabs del editor siguen usando button nativo.
 * sentinel-disable-file html-nativo-en-vez-de-componente: solo aplica a tabs del editor aún. */
import React, {useState, useCallback} from 'react';
import {Star} from 'lucide-react';
import {Modal} from '../ui/Modal';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';
import {Button} from '../ui/Button';
import {SlugInput} from '../ui/SlugInput';
import {UploadImage} from '../ui/UploadImage';
import {RichTextEditor} from '../ui/RichTextEditor';
import {useEditorBlog} from '../../hooks/useEditorBlog';
import type {AdminBlogPost, CreateBlogPostBody, UpdateBlogPostBody} from '../../api/admin-blog';
import './EditorBlog.css';

type TabEditor = 'contenido' | 'media' | 'seo';

interface EditorBlogProps {
    abierto: boolean;
    onCerrar: () => void;
    post: AdminBlogPost | null;
    onGuardar: (body: CreateBlogPostBody | UpdateBlogPostBody) => Promise<void>;
    guardando: boolean;
}

const TABS: {id: TabEditor; label: string}[] = [
    {id: 'contenido', label: 'Contenido'},
    {id: 'media', label: 'Media'},
    {id: 'seo', label: 'SEO'},
];

export const EditorBlog: React.FC<EditorBlogProps> = ({
    abierto,
    onCerrar,
    post,
    onGuardar,
    guardando,
}) => {
    const [tab, setTab] = useState<TabEditor>('contenido');
    const form = useEditorBlog(post, abierto);

    const handleGuardar = useCallback(async () => {
        await onGuardar(form.buildBody());
    }, [form, onGuardar]);

    /* Manejo de tags: campo de texto separados por comas */
    const handleTagsChange = useCallback((valor: string) => {
        const parsed = valor.split(',').map(t => t.trim()).filter(Boolean);
        form.setTags(parsed);
    }, [form]);

    return (
        <Modal abierto={abierto} onCerrar={onCerrar} className="editorBlogModal modalSinPadding">
            <div className="editorBlogTabs">
                {TABS.map(t => (
                    <button
                        key={t.id}
                        type="button"
                        className={`editorBlogTab ${tab === t.id ? 'editorBlogTab--activo' : ''}`}
                        onClick={() => setTab(t.id)}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            <div className="editorBlogCuerpo">
                {tab === 'contenido' && (
                    <div className="editorBlogSeccion">
                        <label className="editorBlogLabel">
                            Título
                            <Input
                                value={form.titulo}
                                onChange={e => form.setTitulo(e.target.value)}
                                placeholder="Título del artículo"
                            />
                        </label>

                        <SlugInput titulo={form.titulo} valor={form.slug} onChange={form.setSlug} />

                        <label className="editorBlogLabel">
                            Extracto
                            <Textarea
                                className="editorBlogTextarea"
                                value={form.extracto}
                                onChange={e => form.setExtracto(e.target.value)}
                                placeholder="Resumen breve del artículo"
                                rows={3}
                            />
                        </label>

                        <label className="editorBlogLabel">
                            Contenido
                        </label>
                        <RichTextEditor
                            contenido={form.contenido}
                            onChange={form.setContenido}
                            placeholder="Escribe el contenido del artículo..."
                        />

                        <label className="editorBlogLabel">
                            Tags (separados por comas)
                            <Input
                                value={form.tags.join(', ')}
                                onChange={e => handleTagsChange(e.target.value)}
                                placeholder="IA, desarrollo web, branding"
                            />
                        </label>

                        {/* [124A-PROJ] Toggle destacado: controla si el post aparece en el inicio */}
                        <Button
                            type="button"
                            variante="outline"
                            tamano="pequeno"
                            className={`editorBlogStarBtn ${form.isFeatured ? 'editorBlogStarBtn--activo' : ''}`}
                            onClick={() => form.setIsFeatured(!form.isFeatured)}
                            title={form.isFeatured ? 'Quitar de inicio' : 'Mostrar en inicio'}
                        >
                            <Star size={16} fill={form.isFeatured ? 'currentColor' : 'none'} />
                            {form.isFeatured ? 'Aparece en inicio' : 'No aparece en inicio'}
                        </Button>
                    </div>
                )}

                {tab === 'media' && (
                    <div className="editorBlogSeccion">
                        <UploadImage
                            valor={form.imagenUrl}
                            onChange={form.setImagenUrl}
                            etiqueta="Imagen destacada"
                        />
                    </div>
                )}

                {tab === 'seo' && (
                    <div className="editorBlogSeccion">
                        <label className="editorBlogLabel">
                            Meta título
                            <Input
                                value={form.metaTitle}
                                onChange={e => form.setMetaTitle(e.target.value)}
                                placeholder="Título para buscadores"
                                maxLength={70}
                            />
                        </label>

                        <label className="editorBlogLabel">
                            Meta descripción
                            <Textarea
                                className="editorBlogTextarea"
                                value={form.metaDescription}
                                onChange={e => form.setMetaDescription(e.target.value)}
                                placeholder="Descripción para buscadores (max 160 caracteres)"
                                rows={3}
                                maxLength={160}
                            />
                        </label>

                        <div className="editorBlogSeoPreview">
                            <span className="editorBlogSeoPreviewLabel">Vista previa en Google</span>
                            <div className="editorBlogSeoCard">
                                <span className="editorBlogSeoTitulo">
                                    {form.metaTitle || form.titulo || 'Título del artículo'}
                                </span>
                                <span className="editorBlogSeoUrl">
                                    nakomi.studio/blog/{form.slug || 'slug'}
                                </span>
                                <span className="editorBlogSeoDesc">
                                    {form.metaDescription || form.extracto || 'Descripción del artículo...'}
                                </span>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            <div className="editorBlogFooter">
                <div className="editorBlogEstado">
                    <Button
                        type="button"
                        variante="outline"
                        tamano="pequeno"
                        className={form.status === 'draft' ? 'editorBlogStatusBtn--activo' : ''}
                        onClick={() => form.setStatus('draft')}
                    >
                        Borrador
                    </Button>
                    <Button
                        type="button"
                        variante="outline"
                        tamano="pequeno"
                        className={form.status === 'published' ? 'editorBlogStatusBtn--activo' : ''}
                        onClick={() => form.setStatus('published')}
                    >
                        Publicado
                    </Button>
                </div>
                <Button
                    variante="primario"
                    tamano="pequeno"
                    onClick={handleGuardar}
                    disabled={guardando || !form.titulo.trim() || !form.slug.trim() || !form.contenido.trim()}
                >
                    {guardando ? 'Guardando...' : 'Guardar'}
                </Button>
            </div>
        </Modal>
    );
};
