/* [104A-28] Sección admin para gestionar problemas reportados en órdenes.
 * Lista problemas con filtros por estado, permite resolver o descartar con respuesta. */
import {useState} from 'react';
import {Loader2, AlertCircle, AlertTriangle, CheckCircle, XCircle, Clock} from 'lucide-react';
import {useSeccionProblemas} from '../../hooks/useSeccionProblemas';
import {
    PROBLEM_STATUS_LABELS,
    type ProblemResponse,
    type ProblemStatus,
    type ProblemAction,
} from '../../api/problems';
import {Textarea} from '../ui/Textarea';
import {Button} from '../ui/Button';
import './SeccionProblemas.css';

const FILTROS: {id: ProblemStatus | 'all'; label: string}[] = [
    {id: 'open', label: 'Abiertos'},
    {id: 'in_review', label: 'En revisión'},
    {id: 'resolved', label: 'Resueltos'},
    {id: 'dismissed', label: 'Descartados'},
    {id: 'all', label: 'Todos'},
];

const STATUS_ICONS: Record<ProblemStatus, React.ElementType> = {
    open: AlertTriangle,
    in_review: Clock,
    resolved: CheckCircle,
    dismissed: XCircle,
};

function formatFecha(iso: string): string {
    return new Date(iso).toLocaleDateString('es-ES', {
        day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit',
    });
}

export function SeccionProblemas() {
    const {
        problemas, totalProblemas, isLoading, error,
        filtroEstado, setFiltroEstado,
        resolviendoId, setResolviendoId,
        handleResolver, resolviendo,
    } = useSeccionProblemas();

    const [respuesta, setRespuesta] = useState('');

    const handleAccion = (id: string, action: ProblemAction) => {
        handleResolver(id, action, respuesta);
        setRespuesta('');
    };

    if (isLoading) {
        return (
            <div className="problemasVacio">
                <Loader2 className="problemasSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="problemasError">
                <AlertCircle size={20} />
                <span>{error}</span>
            </div>
        );
    }

    return (
        <div className="problemasContenedor">
            {/* Tabs de filtro */}
            <div className="problemasTabs">
                {FILTROS.map(f => (
                    <Button
                        key={f.id}
                        variante="texto"
                        tamano="pequeno"
                        type="button"
                        className={`problemasTab ${filtroEstado === f.id ? 'problemasTab--activa' : ''}`}
                        onClick={() => setFiltroEstado(f.id)}
                    >
                        {f.label}
                    </Button>
                ))}
            </div>

            {/* Lista de problemas */}
            {problemas.length === 0 ? (
                <div className="problemasVacio">
                    <AlertTriangle size={24} />
                    <p>No hay problemas {filtroEstado !== 'all' ? `con estado "${PROBLEM_STATUS_LABELS[filtroEstado as ProblemStatus]}"` : ''}.</p>
                </div>
            ) : (
                <div className="problemasLista">
                    <p className="problemasConteo">{problemas.length} de {totalProblemas} problemas</p>
                    {problemas.map(p => (
                        <ProblemaCard
                            key={p.id}
                            problema={p}
                            expandido={resolviendoId === p.id}
                            onExpandir={() => setResolviendoId(resolviendoId === p.id ? null : p.id)}
                            respuesta={resolviendoId === p.id ? respuesta : ''}
                            onRespuestaChange={setRespuesta}
                            onAccion={handleAccion}
                            resolviendo={resolviendo && resolviendoId === p.id}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}

/* Componente interno para cada tarjeta de problema */
interface ProblemaCardProps {
    problema: ProblemResponse;
    expandido: boolean;
    onExpandir: () => void;
    respuesta: string;
    onRespuestaChange: (v: string) => void;
    onAccion: (id: string, action: ProblemAction) => void;
    resolviendo: boolean;
}

function ProblemaCard({
    problema, expandido, onExpandir,
    respuesta, onRespuestaChange, onAccion, resolviendo,
}: ProblemaCardProps) {
    const StatusIcon = STATUS_ICONS[problema.status];
    const canResolve = problema.status === 'open' || problema.status === 'in_review';

    return (
        <div className={`problemaCard ${expandido ? 'problemaCard--expandido' : ''}`}>
            <div className="problemaCardHeader" onClick={onExpandir}>
                <div className="problemaCardInfo">
                    <span className={`problemaBadge problemaBadge--${problema.status}`}>
                        <StatusIcon size={12} />
                        {PROBLEM_STATUS_LABELS[problema.status]}
                    </span>
                    <span className="problemaOrden">Orden #{problema.order_number}</span>
                    <span className="problemaFecha">{formatFecha(problema.created_at)}</span>
                </div>
                <span className="problemaReporter">
                    {problema.reporter_name} ({problema.reporter_role})
                </span>
            </div>

            <p className="problemaRazon">{problema.reason}</p>

            {problema.admin_response && (
                <div className="problemaRespuestaAdmin">
                    <strong>Respuesta:</strong> {problema.admin_response}
                </div>
            )}

            {expandido && canResolve && (
                <div className="problemaAcciones">
                    <Textarea
                        value={respuesta}
                        onChange={e => onRespuestaChange(e.target.value)}
                        placeholder="Respuesta al reporte (opcional)"
                        rows={3}
                        disabled={resolviendo}
                    />
                    <div className="problemaAccionesBtns">
                        <Button
                            variante="primario"
                            tamano="pequeno"
                            type="button"
                            onClick={() => onAccion(problema.id, 'resolve')}
                            disabled={resolviendo}
                        >
                            {resolviendo ? 'Resolviendo...' : 'Resolver'}
                        </Button>
                        <Button
                            variante="outline"
                            tamano="pequeno"
                            type="button"
                            onClick={() => onAccion(problema.id, 'dismiss')}
                            disabled={resolviendo}
                        >
                            Descartar
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
