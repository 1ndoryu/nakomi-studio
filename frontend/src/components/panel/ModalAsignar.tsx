/* [T2-assignment] Modal para que el admin asigne una orden a un empleado.
 * Carga lista de empleados desde GET /api/admin/employees,
 * muestra nombre, especialidades y carga de trabajo actual.
 * Al confirmar hace PUT /api/orders/:id/assign/:employeeId. */

import { useState } from 'react';
import { Loader2 } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiListEmployees, apiAssignOrder, type EmployeeListItem } from '../../api/assignment';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import './ModalAsignar.css';

interface ModalAsignarProps {
    orderId: string;
    orderNumber: number;
    abierto: boolean;
    onCerrar: () => void;
    onAsignado: () => void;
}

export function ModalAsignar({ orderId, abierto, onCerrar, onAsignado }: ModalAsignarProps) {
    const [seleccionado, setSeleccionado] = useState<string | null>(null);
    const queryClient = useQueryClient();

    const { data: empleados = [], isLoading } = useQuery<EmployeeListItem[]>({
        queryKey: ['admin-employees'],
        queryFn: apiListEmployees,
        enabled: abierto,
    });

    const asignar = useMutation({
        mutationFn: () => apiAssignOrder(orderId, seleccionado!),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['ordenes'] });
            queryClient.invalidateQueries({ queryKey: ['orden-detalle', orderId] });
            onAsignado();
        },
    });

    return (
        <Modal abierto={abierto} onCerrar={onCerrar}>
            {isLoading ? (
                <div className="modalAsignarCargando">
                    <Loader2 className="modalAsignarSpinner" size={24} />
                </div>
            ) : empleados.length === 0 ? (
                <p className="modalAsignarVacio">No hay empleados disponibles</p>
            ) : (
                <ul className="modalAsignarLista">
                    {empleados.map(emp => (
                        <li key={emp.user_id}>
                            <Button
                                type="button"
                                variante={seleccionado === emp.user_id ? 'secundario' : 'outline'}
                                tamano="pequeno"
                                className="modalAsignarItem"
                                onClick={() => setSeleccionado(emp.user_id)}
                            >
                                <div className="modalAsignarItemInfo">
                                    <span className="modalAsignarNombre">{emp.email}</span>
                                    {emp.specialties.length > 0 && (
                                        <span className="modalAsignarEspecialidades">
                                            {emp.specialties.join(', ')}
                                        </span>
                                    )}
                                </div>
                                <span className="modalAsignarCarga">
                                    {emp.current_orders}/{emp.max_concurrent_orders} órdenes
                                </span>
                            </Button>
                        </li>
                    ))}
                </ul>
            )}

            <div className="modalAcciones">
                <Button
                    variante="primario"
                    tamano="pequeno"
                    disabled={!seleccionado || asignar.isPending}
                    onClick={() => asignar.mutate()}
                >
                    {asignar.isPending ? <Loader2 className="modalAsignarSpinner" size={14} /> : 'Confirmar asignación'}
                </Button>
                <Button variante="texto" tamano="pequeno" onClick={onCerrar}>
                    Cancelar
                </Button>
                {asignar.isError && (
                    <span className="modalAsignarError">Error al asignar</span>
                )}
            </div>
        </Modal>
    );
}
