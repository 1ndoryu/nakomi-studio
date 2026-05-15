import {useEffect, useRef, useState} from 'react';
import {Image as ImageIcon} from 'lucide-react';
import OptimizedImage from '../ui/OptimizedImage';
import {MenuContextual} from '../ui/ContextMenu';
import {useAuthStore} from '../../stores/authStore';
import '../home/GaleriaHero.css';
import './SolucionHeroImagen.css';

interface SolucionHeroImagenProps {
    src: string;
    alt: string;
    storageKey: string;
}

export function SolucionHeroImagen({src, alt, storageKey}: SolucionHeroImagenProps) {
    const isAdmin = useAuthStore(s => s.user?.effectiveRole === 'admin');
    const inputRef = useRef<HTMLInputElement>(null);
    const [menuAbierto, setMenuAbierto] = useState(false);
    const [previewSrc, setPreviewSrc] = useState<string | null>(null);

    useEffect(() => {
        const cached = window.localStorage.getItem(storageKey);
        if (cached) setPreviewSrc(cached);
    }, [storageKey]);

    const handleArchivo = (event: React.ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        if (!file || !file.type.startsWith('image/')) return;
        const reader = new FileReader();
        reader.onload = () => {
            if (typeof reader.result === 'string') {
                window.localStorage.setItem(storageKey, reader.result);
                setPreviewSrc(reader.result);
            }
        };
        reader.readAsDataURL(file);
    };

    /* [155A-9] Imagen hero editable localmente por admin.
     * Usa el mismo contenedor/optimizador visual que GaleriaHero mientras se conecta persistencia CMS. */
    return (
        <div className="solucionHeroImagenMarco">
            <div className="galeriaHeroContenedor solucionHeroImagenContenedor">
                <OptimizedImage
                    src={previewSrc ?? src}
                    alt={alt}
                    className="galeriaHeroImagen galeriaHeroImagenActiva solucionHeroImagen"
                    fixedWidth={1600}
                    quality={80}
                    loading="lazy"
                    noOptimize={!!previewSrc}
                />
                {isAdmin && (
                    <div className="solucionHeroImagenAcciones">
                        <MenuContextual
                            abierto={menuAbierto}
                            onToggle={() => setMenuAbierto(prev => !prev)}
                            onCerrar={() => setMenuAbierto(false)}
                            ariaLabel="Opciones de imagen"
                            triggerTamano="pequeno"
                            items={[{
                                id: 'cambiar-imagen',
                                label: 'Cambiar imagen',
                                icon: <ImageIcon size={14} />,
                                onSelect: () => inputRef.current?.click(),
                            }]}
                        />
                        <input
                            ref={inputRef}
                            className="solucionHeroImagenInput"
                            type="file"
                            accept="image/*"
                            onChange={handleArchivo}
                        />
                    </div>
                )}
            </div>
        </div>
    );
}
