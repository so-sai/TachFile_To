import React from 'react';

interface FinancialCardProps {
    label: string;
    value: number;
    subValue?: string;
    color?: string;
    isProfit?: boolean;
}

const FinancialCard: React.FC<FinancialCardProps> = ({ label, value, subValue, color = 'var(--white)', isProfit }) => {
    const formatCurrency = (val: number) => {
        return new Intl.NumberFormat('vi-VN', { style: 'currency', currency: 'VND' }).format(val);
    };

    return (
        <div className="brutal-box" style={{ borderColor: isProfit ? color : 'var(--steel)', flex: 1 }}>
            <div style={{ fontSize: '10px', color: '#666', fontWeight: 'bold', letterSpacing: '1px', marginBottom: '8px', textTransform: 'uppercase' }}>
                {label}
            </div>
            <div style={{
                fontSize: isProfit ? '32px' : '24px',
                fontWeight: '900',
                color: color,
                fontFamily: "'Space Mono', monospace",
                whiteSpace: 'nowrap'
            }}>
                {formatCurrency(value)}
            </div>
            {subValue && (
                <div style={{ fontSize: '12px', color: '#888', marginTop: '4px', fontWeight: 'bold' }}>
                    {subValue}
                </div>
            )}
        </div>
    );
};

export default FinancialCard;
