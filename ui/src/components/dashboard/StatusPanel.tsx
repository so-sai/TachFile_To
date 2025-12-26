import React from 'react';

interface StatusPanelProps {
    status: string; // "XANH" | "VÀNG" | "ĐỎ"
    reason: string;
    mock?: boolean;
}

const StatusPanel: React.FC<StatusPanelProps> = ({ status, reason, mock }) => {
    const getStatusColor = () => {
        switch (status) {
            case 'XANH': return 'var(--neon-green)';
            case 'VÀNG': return 'var(--neon-yellow)';
            case 'ĐỎ': return 'var(--neon-red)';
            default: return 'var(--white)';
        }
    };

    const getStatusText = () => {
        switch (status) {
            case 'XANH': return 'AN TOÀN';
            case 'VÀNG': return 'CẢNH BÁO';
            case 'ĐỎ': return 'NGUY HIỂM';
            default: return 'CHƯA XÁC ĐỊNH';
        }
    };

    const color = getStatusColor();

    return (
        <div className="brutal-box" style={{ borderColor: color, minHeight: '180px', display: 'flex', alignItems: 'center', gap: '24px' }}>
            {/* TRAFFIC LIGHT SIGNAL */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'ĐỎ' ? 'var(--neon-red)' : '#111', boxShadow: status === 'ĐỎ' ? '0 0 15px var(--neon-red)' : 'none' }}></div>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'VÀNG' ? 'var(--neon-yellow)' : '#111', boxShadow: status === 'VÀNG' ? '0 0 15px var(--neon-yellow)' : 'none' }}></div>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'XANH' ? 'var(--neon-green)' : '#111', boxShadow: status === 'XANH' ? '0 0 15px var(--neon-green)' : 'none' }}></div>
            </div>

            <div style={{ flex: 1 }}>
                <div style={{ fontSize: '10px', color: '#666', fontWeight: 'bold', letterSpacing: '2px' }}>
                    TRẠNG THÁI DỰ ÁN {mock && '(MÔ PHỎNG)'}
                </div>
                <div style={{ fontSize: '64px', fontWeight: '900', color: color, lineHeight: '1', margin: '4px 0' }}>
                    {getStatusText()}
                </div>
                <div style={{ fontSize: '14px', fontWeight: 'bold', color: '#AAA', textTransform: 'uppercase' }}>
                    {reason}
                </div>
            </div>

            {mock && (
                <div style={{ position: 'absolute', top: '8px', right: '8px', fontSize: '10px', color: 'var(--neon-blue)', border: '1px solid var(--neon-blue)', padding: '2px 4px' }}>
                    MOCK_DATA
                </div>
            )}
        </div>
    );
};

export default StatusPanel;
