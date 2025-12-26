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

// MOCK DATA FOR FOUNDER'S VERIFICATION
const MOCK_DATA: DashboardSummary = {
    status: "ĐỎ",
    status_reason: "SAI LỆCH KHỐI LƯỢNG > 15% & LỢI NHUẬN < 0%",
    top_risks: [
        { description: "BÊ TÔNG MÓNG M250", deviation: 18.5, impact: "-450.000.000 ₫" },
        { description: "CỐT THÉP CỘT D20", deviation: 12.2, impact: "-120.000.000 ₫" },
        { description: "NHÂN CÔNG XÂY TƯỜNG", deviation: 9.8, impact: "-85.000.000 ₫" },
        { description: "ĐÀO ĐẤT HỐ MÓNG", deviation: 22.1, impact: "-310.000.000 ₫" },
        { description: "VẬN CHUYỂN PHẾ THẢI", deviation: 15.0, impact: "-45.000.000 ₫" },
    ],
    payment_progress: {
        received: 12500000000,
        total_contract: 25000000000,
        percent: 50,
        projected_profit: -120000000,
        profit_percent: -0.5
    },
    pending_actions: [
        "KIỂM TRA LẠI ĐƠN GIÁ BÊ TÔNG",
        "RÀ SOÁT KHỐI LƯỢNG ĐÀO ĐẤT",
        "TẠM DỪNG THANH TOÁN ĐỢT 3"
    ],
    metrics: {
        total_rows: 12450,
        total_amount: 25000000000,
        avg_deviation: 15.4,
        high_risk_count: 5,
        critical_count: 3,
        profit_margin_percent: -0.5,
        last_updated: new Date().toISOString()
    }
};

const DashboardFounder: React.FC = () => {
    const [summary, setSummary] = useState<DashboardSummary | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [useMock, setUseMock] = useState(true); // Toggle for verification

    const loadDashboard = async () => {
        if (useMock) {
            setSummary(MOCK_DATA);
            return;
        }

        setLoading(true);
        setError(null);
        try {
            const result = await invoke<DashboardSummary>('get_dashboard_summary');
            setSummary(result);
        } catch (err) {
            console.warn("Could not load real data, falling back to mock", err);
            setSummary(MOCK_DATA);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadDashboard();
    }, [useMock]);

    if (loading) {
        return (
            <div className="cockpit-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <div className="matrix-text animate-pulse" style={{ fontSize: '24px' }}>
                    &gt; ĐANG TRÍCH XUẤT DỮ LIỆU TỪ LÕI THÉP...
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

    if (!summary) return null;

    return (
        <div className="cockpit-container">
            {/* SIDEBAR: VERDICT & VAULT */}
            <div className="sidebar">
                {/* THE VERDICT */}
                <StatusPanel
                    status={summary.status}
                    reason={summary.status_reason}
                    mock={useMock}
                />

                {/* THE VAULT */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', flex: 1 }}>
                    <FinancialCard
                        label="Tổng Doanh Thu (Hợp đồng)"
                        value={summary.payment_progress.total_contract}
                    />
                    <FinancialCard
                        label="Đã Thu Hồi (Thanh toán)"
                        value={summary.payment_progress.received}
                        subValue={`Tiến độ: ${summary.payment_progress.percent}%`}
                    />
                    <FinancialCard
                        label="Lợi Nhuận Dự Kiến (V2.5 logic)"
                        value={summary.payment_progress.projected_profit}
                        subValue={`Tỷ suất: ${summary.payment_progress.profit_percent.toFixed(1)}%`}
                        color={summary.payment_progress.profit_percent > 10 ? 'var(--neon-green)' : (summary.payment_progress.profit_percent > 0 ? 'var(--neon-yellow)' : 'var(--neon-red)')}
                        isProfit={true}
                    />

                    {/* MOCK TOGGLE FOOTER */}
                    <div style={{ marginTop: 'auto', padding: '10px 0' }}>
                        <button
                            onClick={() => setUseMock(!useMock)}
                            style={{
                                width: '100%',
                                background: 'none',
                                border: '2px solid var(--steel)',
                                color: '#444',
                                fontSize: '10px',
                                fontWeight: 'bold',
                                cursor: 'pointer',
                                padding: '4px'
                            }}
                        >
                            {useMock ? 'KÍCH HOẠT DỮ LIỆU THẬT' : 'CHUYỂN SANG MÔ PHỎNG'}
                        </button>
                    </div>
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
