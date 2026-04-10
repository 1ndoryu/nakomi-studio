import React, {useCallback, useEffect, useState} from 'react';
import {Button} from '../ui/Button';
import {Textarea} from '../ui/Textarea';
import {toast} from '../../stores/toastStore';

interface OrderProjectDescriptionProps {
    orderId: string;
    projectDescription: string | null;
    canEdit: boolean;
    isSaving: boolean;
    onSave: (orderId: string, projectDescription: string) => Promise<void>;
}

export const OrderProjectDescription: React.FC<OrderProjectDescriptionProps> = ({
    orderId,
    projectDescription,
    canEdit,
    isSaving,
    onSave,
}) => {
    const [editando, setEditando] = useState(false);
    const [draft, setDraft] = useState(projectDescription ?? '');

    useEffect(() => {
        setDraft(projectDescription ?? '');
        setEditando(false);
    }, [orderId, projectDescription]);

    const guardar = useCallback(async () => {
        const trimmed = draft.trim();
        if (trimmed.length < 10) {
            toast.error('La descripción del proyecto debe tener al menos 10 caracteres.');
            return;
        }

        try {
            await onSave(orderId, trimmed);
            setEditando(false);
            toast.success('Descripción del proyecto actualizada.');
        } catch (err) {
            const message = err instanceof Error ? err.message : 'No se pudo guardar la descripción.';
            toast.error(message);
        }
    }, [draft, onSave, orderId]);

    return (
        <div className="ordenDescripcionProyecto">
            <div className="ordenDescripcionProyectoHeader">
                <span className="ordenInfoLabel">Descripción del proyecto</span>
                {canEdit && !editando && (
                    <Button
                        type="button"
                        variante="texto"
                        tamano="pequeno"
                        onClick={() => setEditando(true)}
                    >
                        Editar
                    </Button>
                )}
            </div>

            {editando ? (
                <div className="ordenDescripcionProyectoEditor">
                    <Textarea
                        className="ordenDescripcionProyectoInput"
                        value={draft}
                        onChange={event => setDraft(event.target.value)}
                        rows={5}
                        placeholder="Describe objetivos, alcance, plazos y cualquier requisito importante del proyecto."
                    />
                    <div className="ordenDescripcionProyectoAcciones">
                        <Button
                            type="button"
                            variante="outline"
                            tamano="pequeno"
                            onClick={() => {
                                setDraft(projectDescription ?? '');
                                setEditando(false);
                            }}
                            disabled={isSaving}
                        >
                            Cancelar
                        </Button>
                        <Button
                            type="button"
                            variante="primario"
                            tamano="pequeno"
                            onClick={() => void guardar()}
                            disabled={isSaving}
                        >
                            {isSaving ? 'Guardando...' : 'Guardar descripción'}
                        </Button>
                    </div>
                </div>
            ) : (
                <p className="ordenDescripcionProyectoTexto">
                    {projectDescription || 'Aún no se cargó una descripción del proyecto.'}
                </p>
            )}
        </div>
    );
};