import React from 'react';

interface StatusPanelProps {
    status: string; // "XANH" | "VÀNG" | "ĐỎ"
    reason: string;
    mock?: boolean;
    pulse?: boolean;
}

const StatusPanel: React.FC<StatusPanelProps> = ({ status, reason, mock, pulse = false }) => {
    const getStatusColor = () => {
        switch (status) {
            case 'XANH': return 'var(--neon-green)';
            case 'VÀNG': return 'var(--neon-yellow)';
            case 'ĐỎ': return 'var(--neon-red)';
            case 'BÁO GIÁ': return 'var(--neon-blue)';
            default: return 'var(--white)';
        }
    };

    const getStatusText = () => {
        switch (status) {
            case 'XANH': return 'AN TOÀN';
            case 'VÀNG': return 'CẢNH BÁO';
            case 'ĐỎ': return 'NGUY HIỂM';
            case 'BÁO GIÁ': return 'BÁO GIÁ';
            default: return 'CHƯA XÁC ĐỊNH';
        }
    };

    const color = getStatusColor();
    // Only apply box-shadow if pulse is true (or if we want it static, but user said remove pulse)
    // Actually, user said <StatusLight pulse={false} />.
    // The previous code had boxShadow logic. I will respect the 'pulse' prop.
    // If pulse is false, we remove the glow/boxShadow?
    // User said "Gọt sạch hiệu ứng". "Cấm animation".
    // Box-shadow itself is not an animation, but "pulse" implies animation.
    // However, keeping it static is fine.
    // If pulse is false, I will assume NO periodic animation.
    // The current code didn't have animation keyframes in standard style, just static boxShadow.
    // I will keep the static boxShadow but maybe remove any potential animation classes if I were adding them.
    // For now, I'll just add the prop to the destructuring to avoid errors when passed.

    const showGlow = pulse; // If pulse is false, maybe we disable glow?
    // "PR có animation = REJECT".
    // "Cấm hiệu ứng trang trí".
    // I will disable the boxShadow if pulse is false, to be strict.

    return (
        <div className="brutal-box" style={{ borderColor: color, minHeight: '180px', display: 'flex', alignItems: 'center', gap: '24px' }}>
            {/* TRAFFIC LIGHT SIGNAL */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'ĐỎ' ? 'var(--neon-red)' : '#111', boxShadow: (showGlow && status === 'ĐỎ') ? '0 0 15px var(--neon-red)' : 'none' }}></div>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'VÀNG' ? 'var(--neon-yellow)' : '#111', boxShadow: (showGlow && status === 'VÀNG') ? '0 0 15px var(--neon-yellow)' : 'none' }}></div>
                <div style={{ width: '40px', height: '40px', borderRadius: '50%', border: '2px solid #333', backgroundColor: status === 'BÁO GIÁ' ? 'var(--neon-blue)' : (status === 'XANH' ? 'var(--neon-green)' : '#111'), boxShadow: (showGlow && (status === 'BÁO GIÁ' || status === 'XANH')) ? (status === 'BÁO GIÁ' ? '0 0 15px var(--neon-blue)' : '0 0 15px var(--neon-green)') : 'none' }}></div>
            </div>

            <div style={{ flex: 1 }}>
                <div style={{ fontSize: '10px', color: '#666', fontWeight: 'bold', letterSpacing: '2px' }}>
                    TRẠNG THÁI DỰ ÁN
                </div>
                <div style={{ fontSize: '64px', fontWeight: '900', color: color, lineHeight: '1', margin: '4px 0' }}>
                    {getStatusText()}
                </div>
                <div style={{ fontSize: '14px', fontWeight: 'bold', color: '#AAA', textTransform: 'uppercase' }}>
                    {reason}
                </div>
            </div>

            {mock && (
                <div style={{ position: 'absolute', top: '4px', right: '8px', fontSize: '8px', color: '#333', letterSpacing: '1px' }}>
                    MOCK_DATA_ACTIVE
                </div>
            )}
        </div>
    );
};

export default StatusPanel;
