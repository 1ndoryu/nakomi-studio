/* [064A-30] FaseCard: card de una fase con entregas, revisiones y acciones.
 * Employee ve botón de extensión de tiempo si la fase está en progreso.
 * Cliente puede aprobar o pedir revisión en fases entregadas. */
import {Check, RotateCcw, CreditCard} from 'lucide-react';
import {
    PHASE_STATUS_LABELS,
    formatPrice,
    type OrderPhaseResponse,
    type PhaseStatus,
} from '../../api/orders';
import {useFaseCard} from '../../hooks/useFaseCard';
import {EntregablesPanel} from './EntregablesPanel';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';

const APPROVABLE_PHASE: PhaseStatus[] = ['delivered'];
const REVISABLE_PHASE: PhaseStatus[] = ['delivered'];

interface FaseCardProps {
    phase: OrderPhaseResponse;
    orderId: string;
    isLast: boolean;
    isClient: boolean;
    isEmployee: boolean;
    isPhased: boolean;
    canDefinePhase: boolean;
    isUpdatingDefinition: boolean;
    onAprobar: (orderId: string, phase: number) => Promise<void>;
    onRevision: (orderId: string, phase: number) => Promise<void>;
    onActualizarFase: (
        orderId: string,
        phase: number,
        req: {
            title?: string;
            description?: string;
            price_cents?: number;
            estimated_days?: number;
            max_revisions?: number;
        },
    ) => Promise<void>;
    onPagarFase: (phaseNumber: number, amountCents: number) => void;
}

export function FaseCard({
    phase,
    orderId,
    isLast,
    isClient,
    isEmployee,
    isPhased,
    canDefinePhase,
    isUpdatingDefinition,
    onAprobar,
    onRevision,
    onActualizarFase,
    onPagarFase,
}: FaseCardProps) {
    const {editando, setEditando, draft, setDraft, canEditDefinition, cancelarEdicion, guardarDefinicion} = useFaseCard({
        phase,
        orderId,
        canDefinePhase,
        onActualizarFase,
    });

    const canApprove = isClient && APPROVABLE_PHASE.includes(phase.status);
    const canRevise = isClient && REVISABLE_PHASE.includes(phase.status) && phase.revisions_used < phase.max_revisions;
    const canDeliver = isEmployee && (phase.status === 'in_progress' || phase.status === 'paid' || phase.status === 'revision_requested');
    const canPayPhase = isClient && isPhased && phase.status === 'pending_payment';
    const isActive = phase.status !== 'locked' && phase.status !== 'approved' && phase.status !== 'skipped';

    const hasDeliverableHistory = ['delivered', 'approved', 'revision_requested', 'in_progress']
        .includes(phase.status) && phase.revisions_used > 0;
    const showDeliverables = canDeliver || hasDeliverableHistory;

    return (
        <div className={`faseCard ${isActive ? 'faseCardActiva' : ''} ${phase.status === 'approved' ? 'faseCardAprobada' : ''}`}>
            <div className="faseTimelineDot">
                <div className={`faseDot ${phase.status === 'approved' ? 'faseDotAprobada' : ''} ${isActive ? 'faseDotActiva' : ''}`} />
                {!isLast && <div className="faseTimelineLine" />}
            </div>

            <div className="faseCardContenido">
                <div className="faseCardHeader">
                    <h4 className="faseNombre">{phase.title}</h4>
                    <div className="faseCardHeaderMeta">
                        <span className={`faseBadge faseBadge--${phase.status.replace('_', '-')}`}>
                            {PHASE_STATUS_LABELS[phase.status]}
                        </span>
                        {canEditDefinition && !editando && (
                            <Button
                                type="button"
                                variante="texto"
                                tamano="pequeno"
                                onClick={() => setEditando(true)}
                            >
                                Definir
                            </Button>
                        )}
                    </div>
                </div>

                {phase.description && <p className="faseDescripcion">{phase.description}</p>}

                {editando && (
                    <div className="faseDefinicionEditor">
                        <label className="faseDefinicionCampo">
                            <span>Título</span>
                            <Input
                                value={draft.title}
                                onChange={event => setDraft(prev => ({...prev, title: event.target.value}))}
                                placeholder="Nombre de la fase"
                            />
                        </label>
                        <label className="faseDefinicionCampo faseDefinicionCampo--full">
                            <span>Descripción</span>
                            <Textarea
                                value={draft.description}
                                onChange={event => setDraft(prev => ({...prev, description: event.target.value}))}
                                rows={3}
                                placeholder="Qué se va a trabajar en esta fase"
                            />
                        </label>
                        <div className="faseDefinicionGrid">
                            <label className="faseDefinicionCampo">
                                <span>Precio USD</span>
                                <Input
                                    type="number"
                                    min="0.01"
                                    step="0.01"
                                    value={draft.priceUsd}
                                    onChange={event => setDraft(prev => ({...prev, priceUsd: event.target.value}))}
                                />
                            </label>
                            <label className="faseDefinicionCampo">
                                <span>Días est.</span>
                                <Input
                                    type="number"
                                    min="1"
                                    step="1"
                                    value={draft.estimatedDays}
                                    onChange={event => setDraft(prev => ({...prev, estimatedDays: event.target.value}))}
                                />
                            </label>
                            <label className="faseDefinicionCampo">
                                <span>Revisiones</span>
                                <Input
                                    type="number"
                                    min="0"
                                    step="1"
                                    value={draft.maxRevisions}
                                    onChange={event => setDraft(prev => ({...prev, maxRevisions: event.target.value}))}
                                />
                            </label>
                        </div>
                        <div className="faseDefinicionAcciones">
                            <Button
                                type="button"
                                variante="outline"
                                tamano="pequeno"
                                onClick={cancelarEdicion}
                                disabled={isUpdatingDefinition}
                            >
                                Cancelar
                            </Button>
                            <Button
                                type="button"
                                variante="primario"
                                tamano="pequeno"
                                onClick={() => void guardarDefinicion()}
                                disabled={isUpdatingDefinition}
                            >
                                {isUpdatingDefinition ? 'Guardando...' : 'Guardar fase'}
                            </Button>
                        </div>
                    </div>
                )}

                <div className="faseMeta">
                    <span>{formatPrice(phase.price_cents)}</span>
                    <span>{phase.estimated_days} días est.</span>
                    <span>Revisiones: {phase.revisions_used}/{phase.max_revisions}</span>
                </div>

                {showDeliverables && (
                    <EntregablesPanel
                        orderId={orderId}
                        phaseNumber={phase.phase_number}
                        canDeliver={canDeliver}
                    />
                )}

                {(canApprove || canRevise || canPayPhase) && (
                    <div className="faseAcciones">
                        {canPayPhase && (
                            <Button
                                className="faseBtn"
                                onClick={() => onPagarFase(phase.phase_number, phase.price_cents)}
                                type="button"
                                variante="secundario"
                                tamano="pequeno"
                            >
                                <CreditCard size={14} /> Pagar fase
                            </Button>
                        )}
                        {canApprove && (
                            <Button
                                className="faseBtn"
                                onClick={() => onAprobar(orderId, phase.phase_number)}
                                type="button"
                                variante="primario"
                                tamano="pequeno"
                            >
                                <Check size={14} /> Aprobar
                            </Button>
                        )}
                        {canRevise && (
                            <Button
                                className="faseBtn"
                                onClick={() => onRevision(orderId, phase.phase_number)}
                                type="button"
                                variante="outline"
                                tamano="pequeno"
                            >
                                <RotateCcw size={14} /> Revisión
                            </Button>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
