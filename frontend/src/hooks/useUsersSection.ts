import {useState} from 'react';
import {useAdminUsers} from './useAdminUsers';
import {useAuthStore} from '../stores/authStore';

type ConfirmUserAction = {
    userId: string;
    type: 'role' | 'status' | 'delete';
    value: string;
    email: string;
} | null;

function getActionErrorMessage(error: unknown, fallback: string): string {
    if (typeof error === 'object' && error !== null) {
        const response = (error as {response?: {data?: {message?: string}}}).response;
        const message = response?.data?.message;
        if (typeof message === 'string' && message.trim()) {
            return message;
        }
    }

    return error instanceof Error ? error.message : fallback;
}

export function useUsersSection() {
    const currentUserId = useAuthStore(s => s.user?.userId ?? null);
    const adminUsers = useAdminUsers();
    const [rolMenuAbierto, setRolMenuAbierto] = useState(false);
    const [statusMenuAbierto, setStatusMenuAbierto] = useState(false);
    const [confirmAction, setConfirmAction] = useState<ConfirmUserAction>(null);
    const [modalError, setModalError] = useState<string | null>(null);

    const isProcessing = adminUsers.isChangingRole
        || adminUsers.isChangingStatus
        || adminUsers.isDeletingUser;

    const openRoleConfirm = (userId: string, email: string, role: string) => {
        setModalError(null);
        setConfirmAction({userId, type: 'role', value: role, email});
    };

    const openStatusConfirm = (userId: string, email: string, status: string) => {
        setModalError(null);
        setConfirmAction({userId, type: 'status', value: status, email});
    };

    const openDeleteConfirm = (userId: string, email: string) => {
        setModalError(null);
        setConfirmAction({userId, type: 'delete', value: '', email});
    };

    const closeConfirm = () => {
        if (isProcessing) return;
        setConfirmAction(null);
        setModalError(null);
    };

    const handleConfirm = async () => {
        if (!confirmAction) return;

        setModalError(null);

        try {
            if (confirmAction.type === 'role') {
                await adminUsers.changeRole({
                    userId: confirmAction.userId,
                    role: confirmAction.value,
                });
            } else if (confirmAction.type === 'status') {
                await adminUsers.changeStatus({
                    userId: confirmAction.userId,
                    status: confirmAction.value,
                });
            } else {
                await adminUsers.deleteUser({userId: confirmAction.userId});
            }

            setConfirmAction(null);
            setModalError(null);
        } catch (err: unknown) {
            setModalError(getActionErrorMessage(err, 'No se pudo completar la acción.'));
        }
    };

    return {
        ...adminUsers,
        currentUserId,
        rolMenuAbierto,
        statusMenuAbierto,
        confirmAction,
        modalError,
        isProcessing,
        setRolMenuAbierto,
        setStatusMenuAbierto,
        openRoleConfirm,
        openStatusConfirm,
        openDeleteConfirm,
        closeConfirm,
        handleConfirm,
    };
}