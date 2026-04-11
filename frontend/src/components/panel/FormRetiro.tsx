/* [T1-withdrawal] Formulario inline para solicitar retiro de saldo.
 * Extraído de SeccionWallet para respetar límite de 300 líneas por componente.
 * Valida monto <= saldo antes de enviar. */

import { useState } from 'react';
import { Loader2 } from 'lucide-react';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Textarea } from '../ui/Textarea';
import { useCreateWithdrawal } from '../../hooks/useWallet';
import './SeccionWallet.css';

interface FormRetiroProps {
    saldoCents: number;
    onClose: () => void;
}

export function FormRetiro({ saldoCents, onClose }: FormRetiroProps) {
    const [monto, setMonto] = useState('');
    const [metodo, setMetodo] = useState('');
    const [detalles, setDetalles] = useState('');
    const crearRetiro = useCreateWithdrawal();

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const centavos = Math.round(parseFloat(monto) * 100);
        if (isNaN(centavos) || centavos <= 0 || centavos > saldoCents) return;
        crearRetiro.mutate(
            {
                amount_cents: centavos,
                payment_method: metodo || undefined,
                payment_details: detalles || undefined,
            },
            {
                onSuccess: () => {
                    onClose();
                },
            }
        );
    }

    return (
        <form className="retiroForm" onSubmit={handleSubmit}>
            <div className="retiroCampo">
                <label className="retiroLabel">Monto (USD)</label>
                <Input
                    type="number"
                    step="0.01"
                    min="0.01"
                    max={(saldoCents / 100).toFixed(2)}
                    value={monto}
                    onChange={e => setMonto(e.target.value)}
                    placeholder="0.00"
                    required
                />
            </div>
            <div className="retiroCampo">
                <label className="retiroLabel">Método de pago</label>
                <Input
                    type="text"
                    value={metodo}
                    onChange={e => setMetodo(e.target.value)}
                    placeholder="PayPal, Zelle, transferencia..."
                />
            </div>
            <div className="retiroCampo">
                <label className="retiroLabel">Detalles (correo, cuenta, etc.)</label>
                <Textarea
                    value={detalles}
                    onChange={e => setDetalles(e.target.value)}
                    placeholder="Indica dónde recibir el pago"
                    rows={2}
                />
            </div>
            <div className="retiroAcciones">
                <Button
                    variante="primario"
                    tamano="pequeno"
                    type="submit"
                    disabled={crearRetiro.isPending}
                >
                    {crearRetiro.isPending ? <Loader2 className="walletSpinner" size={14} /> : 'Enviar solicitud'}
                </Button>
                <Button
                    variante="texto"
                    tamano="pequeno"
                    type="button"
                    onClick={onClose}
                >
                    Cancelar
                </Button>
                {crearRetiro.isError && (
                    <span className="retiroError">Error al crear solicitud</span>
                )}
            </div>
        </form>
    );
}
