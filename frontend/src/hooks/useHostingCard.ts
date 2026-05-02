/* [15A-SENT-1] Hook extraído de HostingCard (HostingSubComponents.tsx): gestiona los 7 useState.
 * Extracción requerida por sentinel usestate-excesivo (max 3 por componente).
 * Gotcha: editPlan/editDomain se inicializan con el sub actual — si el sub cambia externamente
 * (actualización optimista), estos valores no se reflejan hasta que el usuario abre el modal. */

import React, { useState, useCallback } from 'react';
import type { HostingSubscription, UpdateHostingRequest } from '../api/hosting';

interface Opciones {
    sub: HostingSubscription;
    onUpdate: (req: UpdateHostingRequest) => void;
    onAssign?: (email: string) => void;
}

export function useHostingCard({ sub, onUpdate, onAssign }: Opciones) {
    const [menuOpen, setMenuOpen] = useState(false);
    const [editing, setEditing] = useState(false);
    const [showStatusModal, setShowStatusModal] = useState(false);
    const [showAssignModal, setShowAssignModal] = useState(false);
    const [assignEmail, setAssignEmail] = useState('');
    const [editPlan, setEditPlan] = useState(sub.plan);
    const [editDomain, setEditDomain] = useState(sub.domain || '');

    const handleEditSubmit = useCallback(() => {
        onUpdate({ plan: editPlan, domain: editDomain.trim() || undefined });
        setEditing(false);
    }, [editPlan, editDomain, onUpdate]);

    const handleAssignSubmit = useCallback((e: React.FormEvent) => {
        e.preventDefault();
        if (!assignEmail.trim() || !onAssign) { return; }
        onAssign(assignEmail.trim().toLowerCase());
        setShowAssignModal(false);
    }, [assignEmail, onAssign]);

    return {
        menuOpen, setMenuOpen,
        editing, setEditing,
        showStatusModal, setShowStatusModal,
        showAssignModal, setShowAssignModal,
        assignEmail, setAssignEmail,
        editPlan, setEditPlan,
        editDomain, setEditDomain,
        handleEditSubmit,
        handleAssignSubmit,
    };
}
