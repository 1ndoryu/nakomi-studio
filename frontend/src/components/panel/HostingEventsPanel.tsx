/* [074A-63+] EventItem y EventsPanel extraidos de HostingSubComponents.tsx
 * para cumplir limite de 300 lineas. */

import {useQuery} from '@tanstack/react-query';
import {
    apiListHostingEvents,
    type HostingEvent,
} from '../../api/hosting';

function EventItem({event}: {event: HostingEvent}) {
    return (
        <div className="hostingEvento">
            <span className="hostingEventoTipo">{event.event_type}</span>
            <span className="hostingEventoFecha">
                {new Date(event.created_at).toLocaleString('es')}
            </span>
            {event.details && (
                <pre className="hostingEventoDetalles">
                    {JSON.stringify(event.details, null, 2)}
                </pre>
            )}
        </div>
    );
}

export function EventsPanel({
    subscriptionId,
    clientName,
}: {
    subscriptionId: string;
    clientName: string;
}) {
    const {data: events = [], isLoading: cargando} = useQuery({
        queryKey: ['hosting-events', subscriptionId],
        queryFn: () => apiListHostingEvents(subscriptionId),
    });

    return (
        <div className="hostingEventos">
            <h3>Historial: {clientName}</h3>
            {cargando ? (
                <p>Cargando...</p>
            ) : events.length === 0 ? (
                <p className="hostingEventosVacio">Sin eventos registrados</p>
            ) : (
                <div className="hostingEventosLista">
                    {events.map(ev => (
                        <EventItem key={ev.id} event={ev} />
                    ))}
                </div>
            )}
        </div>
    );
}
