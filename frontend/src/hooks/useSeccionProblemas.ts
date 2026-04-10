/* [104A-28] Hook para SeccionProblemas: carga lista de problemas,
 * filtrado por estado, y acciones de resolución/descarte (admin). */
import {useQuery, useMutation, useQueryClient} from '@tanstack/react-query';
import {useState, useMemo, useCallback} from 'react';
import {
    apiListProblems,
    apiResolveProblem,
    type ProblemStatus,
    type ProblemAction,
} from '../api/problems';

type FiltroEstado = ProblemStatus | 'all';

export function useSeccionProblemas() {
    const queryClient = useQueryClient();
    const [filtroEstado, setFiltroEstado] = useState<FiltroEstado>('open');
    const [resolviendoId, setResolviendoId] = useState<string | null>(null);

    const {data: problemas = [], isLoading, error} = useQuery({
        queryKey: ['problems'],
        queryFn: apiListProblems,
    });

    const filtrados = useMemo(() => {
        if (filtroEstado === 'all') return problemas;
        return problemas.filter(p => p.status === filtroEstado);
    }, [problemas, filtroEstado]);

    const resolveMutation = useMutation({
        mutationFn: ({id, action, response}: {id: string; action: ProblemAction; response: string}) =>
            apiResolveProblem(id, {action, response}),
        onSuccess: () => {
            void queryClient.invalidateQueries({queryKey: ['problems']});
            setResolviendoId(null);
        },
    });

    const handleResolver = useCallback((id: string, action: ProblemAction, response: string) => {
        resolveMutation.mutate({id, action, response});
    }, [resolveMutation]);

    return {
        problemas: filtrados,
        totalProblemas: problemas.length,
        isLoading,
        error: error ? String(error) : null,
        filtroEstado,
        setFiltroEstado,
        resolviendoId,
        setResolviendoId,
        handleResolver,
        resolviendo: resolveMutation.isPending,
    };
}
