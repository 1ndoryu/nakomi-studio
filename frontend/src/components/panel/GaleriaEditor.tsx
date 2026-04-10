/* [154A-10] Editor de galería de imágenes del proyecto.
 * Permite subir múltiples imágenes vía /api/admin/uploads y mostrarlas en grid con opción de eliminar.
 * Reutiliza apiUploadImage del módulo de uploads existente.
 * sentinel-disable-file html-nativo-en-vez-de-componente: El botón × de eliminar sobre thumbnail
 * y el input[type=file] oculto no aplican a los componentes UI estándar (Button/Input). */
import React, { useCallback, useRef, useState } from 'react';
import { X } from 'lucide-react';
import { Button } from '../ui/Button';
import { apiUploadImage } from '../../api/uploads';
import './EditorProyecto.css';

interface GaleriaEditorProps {
    galeria: string[];
    onChange: (galeria: string[]) => void;
}

export const GaleriaEditor: React.FC<GaleriaEditorProps> = ({ galeria, onChange }) => {
    const [subiendo, setSubiendo] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    const handleUpload = useCallback(async (files: FileList | null) => {
        if (!files || files.length === 0) return;
        setSubiendo(true);
        const nuevas: string[] = [];
        for (const file of Array.from(files)) {
            if (!file.type.startsWith('image/')) continue;
            try {
                const res = await apiUploadImage(file);
                nuevas.push(res.url);
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

    return (
        <div className="editorProyectoGaleria">
            <span className="editorProyectoLabel">Galería</span>

            {galeria.length > 0 && (
                <div className="editorProyectoGaleriaGrid">
                    {galeria.map((url, idx) => (
                        <div key={`gal-${url}-${idx}`} className="editorProyectoGaleriaItem">
                            <img src={url} alt={`Galería ${idx + 1}`} loading="lazy" />
                            <button
                                type="button"
                                className="editorProyectoGaleriaEliminar"
                                onClick={() => handleRemove(idx)}
                                title="Eliminar imagen"
                            >
                                <X size={14} />
                            </button>
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
