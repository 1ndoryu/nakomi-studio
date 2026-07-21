/* [044A-38 Fase 10] Dashboard admin: métricas, revenue, empleados, alertas.
 * Grid de tarjetas con indicadores clave del marketplace. */

import { RefreshCw, AlertTriangle, DollarSign, ShoppingCart, Users, Star } from 'lucide-react';
import { useDashboard } from '../../hooks/useDashboard';
import { Button } from '../ui/Button';
import './SeccionDashboard.css';

export default function SeccionDashboard() {
  const { dashboard, isLoading, error, refetch } = useDashboard();

  if (isLoading) {
    return <div className="dashboard__loading">Cargando métricas...</div>;
  }

  if (error || !dashboard) {
    return (
      <div className="dashboard__error">
        Error cargando dashboard
        <Button variante="outline" tamano="pequeno" className="dashboard__retry" onClick={() => refetch()}>Reintentar</Button>
      </div>
    );
  }

  const { revenue, orders, employees, alerts } = dashboard;
  const hasAlerts = alerts.unassigned_orders > 0 || alerts.pending_refunds > 0 || alerts.overdue_orders > 0;

  return (
    <div className="dashboard">
      <div className="dashboard__header">
        <h2 className="dashboard__title">Dashboard</h2>
        <Button variante="texto" className="dashboard__refresh" onClick={() => refetch()} title="Actualizar">
          <RefreshCw size={16} />
        </Button>
      </div>

      {/* Alertas */}
      {hasAlerts && (
        <div className="dashboard__alerts">
          <AlertTriangle size={18} />
          <div className="dashboard__alertsList">
            {alerts.unassigned_orders > 0 && (
              <span className="dashboard__alert">{alerts.unassigned_orders} órdenes sin asignar</span>
            )}
            {alerts.pending_refunds > 0 && (
              <span className="dashboard__alert">{alerts.pending_refunds} reembolsos pendientes</span>
            )}
            {alerts.overdue_orders > 0 && (
              <span className="dashboard__alert dashboard__alert--critical">
                {alerts.overdue_orders} órdenes vencidas
              </span>
            )}
          </div>
        </div>
      )}

      {/* Revenue */}
      <div className="dashboard__section">
        <h3 className="dashboard__sectionTitle">
          <DollarSign size={16} /> Revenue
        </h3>
        <div className="dashboard__grid">
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Total</span>
            <span className="dashboard__cardValue">${revenue.total_revenue.toLocaleString('es', { minimumFractionDigits: 2 })}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Este mes</span>
            <span className="dashboard__cardValue">${revenue.monthly_revenue.toLocaleString('es', { minimumFractionDigits: 2 })}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Retenido</span>
            <span className="dashboard__cardValue dashboard__cardValue--held">${revenue.held_amount.toLocaleString('es', { minimumFractionDigits: 2 })}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Reembolsado</span>
            <span className="dashboard__cardValue dashboard__cardValue--refunded">${revenue.refunded_amount.toLocaleString('es', { minimumFractionDigits: 2 })}</span>
          </div>
        </div>
      </div>

      {/* Órdenes */}
      <div className="dashboard__section">
        <h3 className="dashboard__sectionTitle">
          <ShoppingCart size={16} /> Órdenes
        </h3>
        <div className="dashboard__grid">
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Total</span>
            <span className="dashboard__cardValue">{orders.total}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Activas</span>
            <span className="dashboard__cardValue dashboard__cardValue--active">{orders.active}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Completadas</span>
            <span className="dashboard__cardValue dashboard__cardValue--completed">{orders.completed}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Canceladas</span>
            <span className="dashboard__cardValue dashboard__cardValue--cancelled">{orders.cancelled}</span>
          </div>
          <div className="dashboard__card">
            <span className="dashboard__cardLabel">Sin asignar</span>
            <span className="dashboard__cardValue dashboard__cardValue--warning">{orders.awaiting_assignment}</span>
          </div>
        </div>
      </div>

      {/* Empleados */}
      <div className="dashboard__section">
        <h3 className="dashboard__sectionTitle">
          <Users size={16} /> Freelancers
        </h3>
        {employees.length === 0 ? (
          <p className="dashboard__empty">Sin freelancers registrados</p>
        ) : (
          <div className="dashboard__table">
            <div className="dashboard__tableHeader">
              <span>Email</span>
              <span>Activas</span>
              <span>Completadas</span>
              <span>Rating</span>
            </div>
            {employees.map((emp) => (
              <div key={emp.employee_id} className="dashboard__tableRow">
                <span className="dashboard__empEmail">{emp.email}</span>
                <span className="dashboard__empStat">{emp.active_orders}</span>
                <span className="dashboard__empStat">{emp.completed_orders}</span>
                <span className="dashboard__empRating">
                  {emp.average_rating != null ? (
                    <>
                      <Star size={12} fill="#f6ad55" stroke="#f6ad55" />
                      {emp.average_rating.toFixed(1)}
                    </>
                  ) : (
                    '—'
                  )}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
