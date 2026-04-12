/* [154A-10] Editor de galería de imágenes del proyecto.
 * [124A-PROJ1] Soporta GaleriaImagen con layout full/half (1/1 o 1/2 ancho).
 * sentinel-disable-file html-nativo-en-vez-de-componente: El botón × de eliminar sobre thumbnail
 * y el input[type=file] oculto no aplican a los componentes UI estándar (Button/Input). */
import React, { useCallback, useRef, useState } from 'react';
import { X, Maximize2, Columns2 } from 'lucide-react';
import { Button } from '../ui/Button';
import { apiUploadImage } from '../../api/uploads';
import type { GaleriaImagen } from '../../types/contenido';
import './EditorProyecto.css';

interface GaleriaEditorProps {
    galeria: GaleriaImagen[];
    onChange: (galeria: GaleriaImagen[]) => void;
}

export const GaleriaEditor: React.FC<GaleriaEditorProps> = ({ galeria, onChange }) => {
    const [subiendo, setSubiendo] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    const handleUpload = useCallback(async (files: FileList | null) => {
        if (!files || files.length === 0) return;
        setSubiendo(true);
        const nuevas: GaleriaImagen[] = [];
        for (const file of Array.from(files)) {
            if (!file.type.startsWith('image/')) continue;
            try {
                const res = await apiUploadImage(file);
                nuevas.push({ url: res.url, layout: 'full' });
            } catch { /* error manejado por toast futuro */ }
        }
        if (nuevas.length > 0) {
            onChange([...galeria, ...nuevas]);
        }
        setSubiendo(false);
        if (inputRef.current) inputRef.current.value = '';
    }, [galeria, onChange]);

    const handleRemove = useCallback((idx: number) => {
        onChange(galeria.filter((_, i) => i !== idx));
    }, [galeria, onChange]);

    /* [124A-PROJ1] Alternar layout full ↔ half */
    const handleToggleLayout = useCallback((idx: number) => {
        const copia = [...galeria];
        copia[idx] = { ...copia[idx], layout: copia[idx].layout === 'full' ? 'half' : 'full' };
        onChange(copia);
    }, [galeria, onChange]);

    return (
        <div className="editorProyectoGaleria">
            <span className="editorProyectoLabel">Galería</span>

            {galeria.length > 0 && (
                <div className="editorProyectoGaleriaGrid">
                    {galeria.map((img, idx) => (
                        <div key={`gal-${img.url}-${idx}`} className={`editorProyectoGaleriaItem ${img.layout === 'half' ? 'editorProyectoGaleriaItem--half' : ''}`}>
                            <img src={img.url} alt={`Galería ${idx + 1}`} loading="lazy" />
                            <div className="editorProyectoGaleriaControles">
                                <button
                                    type="button"
                                    className="editorProyectoGaleriaLayout"
                                    onClick={() => handleToggleLayout(idx)}
                                    title={img.layout === 'full' ? 'Cambiar a 1/2 ancho' : 'Cambiar a ancho completo'}
                                >
                                    {img.layout === 'full' ? <Maximize2 size={14} /> : <Columns2 size={14} />}
                                    <span>{img.layout === 'full' ? '1/1' : '1/2'}</span>
                                </button>
                                <button
                                    type="button"
                                    className="editorProyectoGaleriaEliminar"
                                    onClick={() => handleRemove(idx)}
                                    title="Eliminar imagen"
                                >
                                    <X size={14} />
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            <input
                ref={inputRef}
                type="file"
                accept="image/*"
                multiple
                className="editorProyectoGaleriaFileInput"
                onChange={e => handleUpload(e.target.files)}
            />
            <Button
                variante="texto"
                tamano="pequeno"
                onClick={() => inputRef.current?.click()}
                disabled={subiendo}
            >
                {subiendo ? 'Subiendo...' : '+ Añadir imágenes'}
            </Button>
        </div>
    );
};
