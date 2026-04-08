/* [074A-6] UploadImage — Componente de subida de imagen con preview.
 * Click o drag-and-drop para seleccionar imagen.
 * Sube al endpoint /api/admin/uploads y retorna URL via onChange. */
import React, { useCallback, useRef, useState } from 'react';
import { Camera, X } from 'lucide-react';
import { apiUploadImage } from '../../api/uploads';
import { Button } from './Button';
import './UploadImage.css';

interface UploadImageProps {
    valor?: string;
    onChange: (url: string) => void;
    etiqueta?: string;
    className?: string;
}

export const UploadImage: React.FC<UploadImageProps> = ({
    valor,
    onChange,
    etiqueta = 'Subir imagen',
    className = '',
}) => {
    const [subiendo, setSubiendo] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [arrastrando, setArrastrando] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    const procesarArchivo = useCallback(async (file: File) => {
        setError(null);
        setSubiendo(true);
        try {
            const res = await apiUploadImage(file);
            onChange(res.url);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Error al subir imagen';
            setError(msg);
        } finally {
            setSubiendo(false);
        }
    }, [onChange]);

    const handleClick = () => {
        inputRef.current?.click();
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file) {
            procesarArchivo(file);
            e.target.value = '';
        }
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setArrastrando(false);
        const file = e.dataTransfer.files[0];
        if (file && file.type.startsWith('image/')) {
            procesarArchivo(file);
        }
    };

    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault();
        setArrastrando(true);
    };

    const handleDragLeave = () => {
        setArrastrando(false);
    };

    const handleRemove = (e: React.MouseEvent) => {
        e.stopPropagation();
        onChange('');
    };

    return (
        <div className={`uploadImagen ${className}`}>
            {etiqueta && <span className="uploadImagenEtiqueta">{etiqueta}</span>}
            <div
                className={`uploadImagenArea ${arrastrando ? 'uploadImagenArrastrando' : ''} ${valor ? 'uploadImagenConPreview' : ''}`}
                onClick={handleClick}
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(); }}
            >
                {valor ? (
                    <div className="uploadImagenPreview">
                        <img src={valor} alt="Preview" loading="lazy" />
                        <Button
                            variante="texto"
                            tamano="pequeno"
                            className="uploadImagenEliminar"
                            onClick={handleRemove}
                            aria-label="Eliminar imagen"
                        >
                            <X size={14} />
                        </Button>
                    </div>
                ) : (
                    <div className="uploadImagenPlaceholder">
                        {subiendo ? (
                            <span className="uploadImagenCargando">Subiendo...</span>
                        ) : (
                            <>
                                <Camera size={24} className="uploadImagenIcono" />
                                <span>Click o arrastra una imagen</span>
                            </>
                        )}
                    </div>
                )}
                <input
                    ref={inputRef}
                    type="file"
                    accept="image/jpeg,image/png,image/webp,image/gif,image/svg+xml"
                    onChange={handleChange}
                    className="uploadImagenInput"
                />
            </div>
            {error && <span className="uploadImagenError">{error}</span>}
        </div>
    );
};
