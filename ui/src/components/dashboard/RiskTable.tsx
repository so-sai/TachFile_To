import React from 'react';

interface RiskItem {
    description: string;
    deviation: number;
    impact: string;
}

interface RiskTableProps {
    risks: RiskItem[];
    smoothScroll?: boolean;
}

const RiskTable: React.FC<RiskTableProps> = ({ risks, smoothScroll = false }) => {
    return (
        <div className="brutal-box" style={{ height: '100%', borderColor: 'var(--steel)' }}>
            <div style={{ fontSize: '12px', fontWeight: 'bold', marginBottom: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span>VECTƠ RỦI RO (TOP 5)</span>
                <span style={{ color: 'var(--neon-red)', fontSize: '10px' }}>PHÁT HIỆN {risks.length} ĐIỂM NÓNG</span>
            </div>

            <table className="iron-table">
                <thead>
                    <tr>
                        <th style={{ color: '#888' }}>HẠNG MỤC</th>
                        <th style={{ textAlign: 'center', color: '#D97706' }}>S.LỆCH</th>
                        <th style={{ textAlign: 'right', color: '#888' }}>TÁC ĐỘNG</th>
                        <th style={{ textAlign: 'center' }}>SOI</th>
                    </tr>
                </thead>
                <tbody>
                    {risks.length > 0 ? risks.map((risk, i) => (
                        <tr key={i}>
                            <td style={{ fontSize: '12px', fontWeight: 'bold', color: '#DDD' }}>{risk.description}</td>
                            <td style={{
                                textAlign: 'center',
                                color: Math.abs(risk.deviation) >= 15 ? 'var(--neon-red)' : 'var(--neon-yellow)',
                                fontWeight: '900'
                            }}>
                                {risk.deviation > 0 ? '+' : ''}{(risk.deviation ?? 0).toFixed(1)}%
                            </td>
                            <td style={{ textAlign: 'right', color: '#888', fontSize: '11px' }}>{risk.impact}</td>
                            <td style={{ textAlign: 'center' }}>
                                <button style={{
                                    background: 'none',
                                    border: '1px solid var(--steel)',
                                    color: 'var(--neon-blue)',
                                    cursor: 'pointer',
                                    fontSize: '10px',
                                    padding: '2px 4px'
                                }}>
                                    🔍
                                </button>
                            </td>
                        </tr>
                    )) : (
                        <tr>
                            <td colSpan={4} style={{ textAlign: 'center', padding: '40px', color: 'var(--neon-green)', fontWeight: 'bold' }}>
                                HỆ THỐNG CHƯA PHÁT HIỆN RỦI RO TRỌNG YẾU
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    );
};

export default RiskTable;
