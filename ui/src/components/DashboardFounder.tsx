import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import '../styles/Dashboard.css';

// Import sub-components
import StatusPanel from './dashboard/StatusPanel';
import FinancialCard from './dashboard/FinancialCard';
import RiskTable from './dashboard/RiskTable';
import ActionPrompt from './dashboard/ActionPrompt';

interface DashboardSummary {
    status: string;
    status_reason: string;
    top_risks: Array<{
        description: string;
        deviation: number;
        impact: string;
    }>;
    payment_progress: {
        received: number;
        total_contract: number;
        percent: number;
        projected_profit: number;
        profit_percent: number;
    };
    pending_actions: string[];
    metrics: {
        total_rows: number;
        total_amount: number;
        avg_deviation: number;
        high_risk_count: number;
        critical_count: number;
        profit_margin_percent: number;
        last_updated: string;
    };
}

const DashboardFounder: React.FC = () => {
    const [summary, setSummary] = useState<DashboardSummary | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const loadDashboard = async () => {
        setLoading(true);
        setError(null);
        try {
            const result = await invoke<DashboardSummary>('get_dashboard_summary');
            setSummary(result);
        } catch (err: any) {
            console.warn("Could not load real data", err);
            setError(err.toString());
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadDashboard();
    }, []);

    if (loading) {
        return (
            <div className="cockpit-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#050505' }}>
                <div className="radar-container">
                    <div className="radar-circle"></div>
                    <div className="radar-sweep"></div>
                    <div className="matrix-text animate-pulse" style={{ fontSize: '18px', marginTop: '120px', letterSpacing: '2px' }}>
                        &gt; ĐANG TRÍCH XUẤT DỮ LIỆU TỪ LÕI THÉP...
                    </div>
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="cockpit-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '40px' }}>
                <div className="brutal-box brutal-box-red w-full">
                    <div className="font-black text-2xl mb-4">LỖI HỆ THỐNG</div>
                    <div className="font-mono">{error}</div>
                </div>
            </div>
        );
    }

    if (!summary || summary.metrics.total_rows === 0) {
        return (
            <div className="cockpit-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#050505' }}>
                <div className="radar-container" style={{ opacity: 0.6 }}>
                    <div className="radar-circle"></div>
                    <div className="radar-sweep" style={{ animationDuration: '4s' }}></div>
                    <div className="matrix-text" style={{ fontSize: '14px', marginTop: '120px', color: 'var(--neon-blue)', letterSpacing: '4px' }}>
                        &gt; CHỜ DỮ LIỆU ĐẦU VÀO [DRAG & DROP]...
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="cockpit-container">
            {/* SIDEBAR: VERDICT & VAULT */}
            <div className="sidebar">
                {/* THE VERDICT */}
                <StatusPanel
                    status={summary.status}
                    reason={summary.status_reason}
                />

                {/* THE VAULT */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', flex: 1 }}>
                    <FinancialCard
                        label="DOANH THU"
                        value={summary.payment_progress.total_contract}
                    />
                    <FinancialCard
                        label="ĐÃ THU"
                        value={summary.payment_progress.received}
                        subValue={`Tiến độ: ${summary.payment_progress.percent.toFixed(1)}%`}
                    />
                    <FinancialCard
                        label="LỢI NHUẬN"
                        value={summary.payment_progress.projected_profit}
                        subValue={`Tỷ suất: ${summary.payment_progress.profit_percent.toFixed(1)}%`}
                        color={summary.payment_progress.profit_percent > 10 ? 'var(--neon-green)' : (summary.payment_progress.profit_percent > 0 ? 'var(--neon-yellow)' : 'var(--neon-red)')}
                        isProfit={true}
                    />
                </div>
            </div>

            {/* THE RISK VECTOR */}
            <div className="main-panel">
                <RiskTable risks={summary.top_risks} />
            </div>

            {/* THE ACTION BAR */}
            <div style={{ gridColumn: 'span 2' }}>
                <ActionPrompt actions={summary.pending_actions} />
            </div>
        </div>
    );
};

export default DashboardFounder;
