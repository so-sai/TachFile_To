import React from 'react';

interface StatusBadgeProps {
    status: 'valid' | 'warning' | 'error';
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
    const config = {
        valid: {
            text: '✓ Hợp lệ',
            className: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border border-green-200 dark:border-green-800',
        },
        warning: {
            text: '⚠ Cảnh báo',
            className: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 border border-yellow-200 dark:border-yellow-800',
        },
        error: {
            text: '✗ Lỗi',
            className: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 border border-red-200 dark:border-red-800',
        },
    };

    const { text, className } = config[status];

    return (
        <div className={`inline-flex items-center justify-center px-3 py-1 rounded-full text-xs font-medium ${className}`}>
            {text}
        </div>
    );
};
