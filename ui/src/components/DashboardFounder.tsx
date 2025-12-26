import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface DashboardSummary {
    status: string;
    status_reason: string;
    top_risks: Array<{
        description: string;
        value: string;
        severity: string;
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
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
            console.error('Dashboard error:', err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadDashboard();
    }, []);

    if (loading) {
        return (
            <div className="flex items-center justify-center h-64">
                <div className="text-center">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                    <p className="mt-4 text-gray-600">Đang phân tích dữ liệu...</p>
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
                <h3 className="text-red-800 font-semibold mb-2">Không thể tải dashboard</h3>
                <p className="text-red-600 mb-4">{error}</p>
                <button
                    onClick={loadDashboard}
                    className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition"
                >
                    Thử lại
                </button>
            </div>
        );
    }

    if (!summary) {
        return (
            <div className="text-center p-8 text-gray-500">
                <p>Chưa có dữ liệu. Vui lòng tải file Excel trước.</p>
            </div>
        );
    }

    const statusColor = summary.status === "XANH"
        ? "text-green-700 bg-green-50 border-green-200"
        : summary.status === "VÀNG"
            ? "text-yellow-700 bg-yellow-50 border-yellow-200"
            : "text-red-700 bg-red-50 border-red-200";

    return (
        <div className="space-y-6 p-6">
            {/* HEADER */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900">Dashboard Founder</h1>
                    <p className="text-gray-600">Phân tích dự án theo thời gian thực</p>
                </div>
                <div className="text-sm text-gray-500">
                    Cập nhật: {new Date(summary.metrics.last_updated).toLocaleTimeString('vi-VN')}
                </div>
            </div>

            {/* MAIN GRID */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {/* 1. ĐÈN TỔNG QUAN */}
                <div className={`col-span-1 md:col-span-2 border-2 rounded-xl p-6 flex flex-col items-center justify-center ${statusColor}`}>
                    <div className="text-5xl font-bold mb-3">{summary.status}</div>
                    <div className="text-center text-sm mb-4">{summary.status_reason}</div>
                    <div className="grid grid-cols-3 gap-4 w-full mt-4">
                        <div className="text-center">
                            <div className="text-2xl font-semibold">{summary.metrics.total_rows}</div>
                            <div className="text-xs text-gray-600">Hạng mục</div>
                        </div>
                        <div className="text-center">
                            <div className="text-2xl font-semibold">{summary.metrics.high_risk_count}</div>
                            <div className="text-xs text-gray-600">Rủi ro cao</div>
                        </div>
                        <div className="text-center">
                            <div className="text-2xl font-semibold">{summary.metrics.avg_deviation.toFixed(1)}%</div>
                            <div className="text-xs text-gray-600">Lệch TB</div>
                        </div>
                    </div>
                </div>

                {/* 2. TOP RỦI RO */}
                <div className="bg-white border-2 border-gray-300 rounded-xl p-6">
                    <h2 className="text-lg font-semibold mb-4 flex items-center">
                        <span className="w-3 h-3 bg-red-500 rounded-full mr-2"></span>
                        CẢNH BÁO NGAY
                    </h2>
                    <ul className="space-y-3">
                        {summary.top_risks.map((risk, i) => (
                            <li key={i} className="flex justify-between items-start p-3 bg-gray-50 rounded">
                                <div className="flex-1">
                                    <span className="text-sm block mb-1">{risk.description}</span>
                                    <span className={`text-xs px-2 py-1 rounded inline-block ${risk.severity === 'HIGH' ? 'bg-red-100 text-red-800' : 'bg-yellow-100 text-yellow-800'}`}>
                                        {risk.severity}
                                    </span>
                                </div>
                            </li>
                        ))}
                    </ul>
                </div>

                {/* 3. TIẾN ĐỘ THANH TOÁN */}
                <div className="bg-white border-2 border-gray-300 rounded-xl p-6">
                    <h2 className="text-lg font-semibold mb-4">💰 THANH TOÁN</h2>
                    <div className="mb-4">
                        <div className="flex justify-between mb-1">
                            <span className="text-sm">Tiến độ</span>
                            <span className="font-semibold">{summary.payment_progress.percent.toFixed(1)}%</span>
                        </div>
                        <div className="w-full bg-gray-200 rounded-full h-2">
                            <div
                                className="bg-blue-600 h-2 rounded-full"
                                style={{ width: `${Math.min(100, summary.payment_progress.percent)}%` }}
                            ></div>
                        </div>
                    </div>
                    <div className="space-y-2 text-sm">
                        <div className="flex justify-between">
                            <span>Đã thu:</span>
                            <span className="font-medium">{(summary.payment_progress.received / 1e6).toFixed(0)}M</span>
                        </div>
                        <div className="flex justify-between">
                            <span>Tổng HĐ:</span>
                            <span className="font-medium">{(summary.payment_progress.total_contract / 1e6).toFixed(0)}M</span>
                        </div>
                        <div className={`flex justify-between pt-2 border-t ${summary.payment_progress.projected_profit >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                            <span className="font-semibold">Lãi DK:</span>
                            <span className="font-bold">
                                {summary.payment_progress.projected_profit >= 0 ? '+' : ''}
                                {(summary.payment_progress.projected_profit / 1e6).toFixed(0)}M
                            </span>
                        </div>
                    </div>
                </div>

                {/* 4. HÀNH ĐỘNG */}
                <div className="bg-white border-2 border-gray-300 rounded-xl p-6">
                    <h2 className="text-lg font-semibold mb-4">📋 VIỆC CẦN LÀM</h2>
                    <ul className="space-y-3">
                        {summary.pending_actions.map((action, i) => (
                            <li key={i} className="flex items-start gap-3 p-3 bg-blue-50 rounded border border-blue-100">
                                <div className="flex-shrink-0 w-6 h-6 bg-blue-500 text-white rounded-full flex items-center justify-center text-sm">
                                    {i + 1}
                                </div>
                                <span className="text-sm">{action}</span>
                            </li>
                        ))}
                    </ul>
                    <button
                        onClick={loadDashboard}
                        className="w-full mt-6 px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-900 transition text-sm"
                    >
                        🔄 Cập nhật
                    </button>
                </div>
            </div>

            {/* QUICK STATS */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <div className="text-2xl font-bold">{(summary.metrics.total_amount / 1e6).toFixed(0)}M</div>
                    <div className="text-sm text-gray-600">Tổng giá trị</div>
                </div>
                <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <div className="text-2xl font-bold">{summary.metrics.avg_deviation.toFixed(1)}%</div>
                    <div className="text-sm text-gray-600">Lệch TB</div>
                </div>
                <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <div className="text-2xl font-bold">{summary.metrics.high_risk_count}</div>
                    <div className="text-sm text-gray-600">Rủi ro</div>
                </div>
                <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <div className="text-2xl font-bold">{summary.payment_progress.percent.toFixed(0)}%</div>
                    <div className="text-sm text-gray-600">Thanh toán</div>
                </div>
            </div>
        </div>
    );
};

export default DashboardFounder;
