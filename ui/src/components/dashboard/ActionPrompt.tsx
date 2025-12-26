import React from 'react';

interface ActionPromptProps {
    actions: string[];
}

const ActionPrompt: React.FC<ActionPromptProps> = ({ actions }) => {
    return (
        <div className="brutal-box" style={{
            backgroundColor: '#050505',
            borderColor: 'var(--neon-blue)',
            display: 'flex',
            alignItems: 'center',
            gap: '16px',
            padding: '8px 16px'
        }}>
            <div style={{ color: 'var(--neon-blue)', fontWeight: 'black', fontSize: '18px', fontFamily: 'Space Mono' }}>
                &gt;
            </div>

            <div style={{ flex: 1, display: 'flex', gap: '12px', overflow: 'hidden' }}>
                {actions.map((action, i) => (
                    <div key={i} className="matrix-text" style={{ whiteSpace: 'nowrap', textTransform: 'uppercase' }}>
                        {action}{i < actions.length - 1 ? ' | ' : ''}
                    </div>
                ))}
            </div>

            <div style={{ display: 'flex', gap: '8px' }}>
                <button style={{
                    backgroundColor: 'var(--neon-blue)',
                    color: 'var(--black)',
                    border: 'none',
                    fontWeight: 'black',
                    fontSize: '10px',
                    padding: '4px 12px',
                    cursor: 'pointer'
                }}>
                    XUẤT BÁO CÁO
                </button>
            </div>
        </div>
    );
};

export default ActionPrompt;
