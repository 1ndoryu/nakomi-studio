/* [044A-43] Hook de perfil: carga perfil desde backend, sube avatar.
 * [074A-23] handleGuardar conectado a PATCH /api/profile (antes era stub). */
import {useState, useEffect} from 'react';
import {useQueryClient} from '@tanstack/react-query';
import {subirAvatar, actualizarPerfil} from '../api/profile';
import type {PerfilResponse} from '../api/profile';
import {currentProfileKey, useCurrentProfile} from './useCurrentProfile';
import {useAuthStore} from '../stores/authStore';

interface EstadoPerfil {
    email: string;
    nombre: string;
    descripcion: string;
    linkedin: string;
    twitter: string;
    website: string;
}

interface RetornoUsePerfil {
    estado: EstadoPerfil;
    guardado: boolean;
    guardando: boolean;
    errorGuardar: string | null;
    cargando: boolean;
    perfil: PerfilResponse | null;
    avatarUrl: string;
    subiendoAvatar: boolean;
    actualizarCampo: (campo: keyof EstadoPerfil, valor: string) => void;
    handleGuardar: (e: React.FormEvent) => void;
    handleSubirAvatar: (archivo: File) => Promise<void>;
}

export const usePerfil = (): RetornoUsePerfil => {
    const queryClient = useQueryClient();
    const userId = useAuthStore(s => s.user?.userId);
    const actualizarEmailAuth = useAuthStore(s => s.actualizarEmail);
    const {perfil, cargando: cargandoPerfil, avatarUrl} = useCurrentProfile();
    const [subiendoAvatar, setSubiendoAvatar] = useState(false);

    const [estado, setEstado] = useState<EstadoPerfil>({
        email: '',
        nombre: '',
        descripcion: '',
        linkedin: '',
        twitter: '',
        website: ''
    });

    const [guardado, setGuardado] = useState(false);
    const [guardando, setGuardando] = useState(false);
    const [errorGuardar, setErrorGuardar] = useState<string | null>(null);

    /* Sincroniza los datos editables cuando el backend devuelve el perfil actual */
    useEffect(() => {
        if (!perfil) return;
        setEstado(prev => ({
            ...prev,
            email: prev.email || perfil.email || '',
            nombre: prev.nombre || perfil.display_name || ''
        }));
    }, [perfil]);

    const actualizarCampo = (campo: keyof EstadoPerfil, valor: string) => {
        setEstado(prev => ({...prev, [campo]: valor}));
    };

    /* [074A-23] Guardar perfil llamando PATCH /api/profile */
    const handleGuardar = async (e: React.FormEvent) => {
        e.preventDefault();
        setGuardando(true);
        setErrorGuardar(null);
        try {
            const resp = await actualizarPerfil({
                email: estado.email.trim() || undefined,
                display_name: estado.nombre || undefined,
                bio: estado.descripcion || undefined,
                linkedin: estado.linkedin || undefined,
                twitter: estado.twitter || undefined,
                website: estado.website || undefined,
            });
            queryClient.setQueryData<PerfilResponse | undefined>(
                currentProfileKey(userId),
                (prev) => prev ? {...prev, email: resp.email, display_name: resp.display_name} : prev,
            );
            actualizarEmailAuth(resp.email);
            setEstado(prev => ({...prev, email: resp.email}));
            setGuardado(true);
            setTimeout(() => setGuardado(false), 3000);
        } catch (err: unknown) {
            const axiosData = (err as { response?: { data?: { message?: string } } })?.response?.data;
            setErrorGuardar(axiosData?.message ?? (err instanceof Error ? err.message : 'Error guardando perfil'));
        } finally {
            setGuardando(false);
        }
    };

    const handleSubirAvatar = async (archivo: File) => {
        setSubiendoAvatar(true);
        try {
            const resp = await subirAvatar(archivo);
            queryClient.setQueryData<PerfilResponse | undefined>(
                currentProfileKey(userId),
                (prev) => prev ? {...prev, avatar_url: resp.avatar_url} : prev,
            );
        } finally {
            setSubiendoAvatar(false);
        }
    };

    return {
        estado, guardado, guardando, errorGuardar, cargando: cargandoPerfil, perfil, avatarUrl,
        subiendoAvatar, actualizarCampo, handleGuardar, handleSubirAvatar
    };
};
