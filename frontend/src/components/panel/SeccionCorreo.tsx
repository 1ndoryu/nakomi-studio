/* [311A-1] Sección admin para visualizar trazabilidad de correos enviados.
 * Lista paginada con filtro por tipo de plantilla.
 * Sigue el patrón visual de SeccionReembolsos. */

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Loader2, AlertCircle, Mail, ChevronLeft, ChevronRight, Filter } from 'lucide-react';
import { apiListEmailLogs, TEMPLATE_OPTIONS, type EmailLogItem } from '../../api/admin-email';
import { Button } from '../ui/Button';
import { Select } from '../ui/Select';
import './SeccionCorreo.css';

const PAGE_SIZE = 50;

export function SeccionCorreo() {
    const [filtroTemplate, setFiltroTemplate] = useState<string>('');
    const [offset, setOffset] = useState(0);

    const { data, isLoading, error } = useQuery({
        queryKey: ['admin-email-logs', filtroTemplate, offset],
        queryFn: () => apiListEmailLogs({
            template: filtroTemplate || undefined,
            limit: PAGE_SIZE,
            offset,
        }),
    });

    const logs = data?.logs ?? [];
    const total = data?.total ?? 0;
    const paginaActual = Math.floor(offset / PAGE_SIZE) + 1;
    const totalPaginas = Math.max(1, Math.ceil(total / PAGE_SIZE));

    const handleFilterChange = (value: string) => {
        setFiltroTemplate(value);
        setOffset(0);
    };

    if (isLoading) {
        return (
            <div className="correosVacio">
                <Loader2 className="correosSpinner" size={32} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="correosError">
                <AlertCircle size={20} />
                <span>Error al cargar correos: {(error as Error).message}</span>
            </div>
        );
    }

    return (
        <div className="correosContenedor">
            <h2 className="correosTitulo">Correos enviados</h2>

            {/* Filtro por plantilla */}
            <div className="correosFiltro">
                <Filter size={18} />
                <Select
                    value={filtroTemplate}
                    onChange={(e) => handleFilterChange(e.target.value)}
                >
                    <option value="">Todas las plantillas</option>
                    {TEMPLATE_OPTIONS.map(opt => (
                        <option key={opt.value} value={opt.value}>{opt.label}</option>
                    ))}
                </Select>
                <span className="correosTotal">{total} correos</span>
            </div>

            {/* Tabla */}
            {logs.length === 0 ? (
                <div className="correosVacio">
                    <Mail size={40} />
                    <p>No hay correos enviados</p>
                </div>
            ) : (
                <div className="correosTablaWrapper">
                    <table className="correosTabla">
                        <thead>
                            <tr>
                                <th>Plantilla</th>
                                <th>Destinatario</th>
                                <th>Asunto</th>
                                <th>Estado</th>
                                <th>Fecha</th>
                            </tr>
                        </thead>
                        <tbody>
                            {logs.map((log: EmailLogItem) => (
                                <tr key={log.id} className="correosFila">
                                    <td>
                                        <span className="correosTag">{log.template_label}</span>
                                    </td>
                                    <td className="correosDestinatario">{log.to_email}</td>
                                    <td className="correosAsunto">{log.subject}</td>
                                    <td>
                                        <span className={`correosBadge ${log.status_color}`}>
                                            {log.status === 'sent' ? 'Enviado' : log.status === 'failed' ? 'Fallido' : log.status}
                                        </span>
                                    </td>
                                    <td className="correosFecha">
                                        {new Date(log.created_at).toLocaleDateString('es-ES', {
                                            year: 'numeric',
                                            month: 'short',
                                            day: 'numeric',
                                            hour: '2-digit',
                                            minute: '2-digit',
                                        })}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}

            {/* Paginación */}
            {total > PAGE_SIZE && (
                <div className="correosPaginacion">
                    <Button
                        variante="texto"
                        tamano="pequeno"
                        type="button"
                        disabled={offset === 0}
                        onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
                    >
                        <ChevronLeft size={16} /> Anterior
                    </Button>
                    <span className="correosPagInfo">
                        Página {paginaActual} de {totalPaginas} ({(paginaActual - 1) * PAGE_SIZE + 1}–{Math.min(paginaActual * PAGE_SIZE, total)} de {total})
                    </span>
                    <Button
                        variante="texto"
                        tamano="pequeno"
                        type="button"
                        disabled={offset + PAGE_SIZE >= total}
                        onClick={() => setOffset(offset + PAGE_SIZE)}
                    >
                        Siguiente <ChevronRight size={16} />
                    </Button>
                </div>
            )}
        </div>
    );
}
