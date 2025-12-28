/**
 * TACHFILETO V3.1 - DISPLAY MAP (Single Source of Truth)
 * =====================================================
 * ASCII System Keys -> Vietnamese Display Labels
 */

export const COLUMN_DISPLAY_MAP: Record<string, string> = {
    // Financial columns
    'thanh_tien': 'THÀNH TIỀN (VNĐ)',
    'don_gia': 'ĐƠN GIÁ (VNĐ)',
    'tong_cong': 'TỔNG CỘNG',

    // Description columns
    'hang_muc': 'HẠNG MỤC / DIỄN GIẢI',
    'dien_giai': 'DIỄN GIẢI',
    'ten': 'TÊN',
    'mo_ta': 'MÔ TẢ',

    // Quantity columns
    'khoi_luong': 'KHỐI LƯỢNG',
    'khoi_luong_tinh_toan': 'KL TÍNH TOÁN',
    'khoi_luong_thuc_te': 'KL THỰC TẾ',
    'so_luong': 'SỐ LƯỢNG',

    // Unit columns
    'don_vi': 'ĐƠN VỊ TÍNH',
    'dvt': 'ĐVT',

    // Status columns
    'trang_thai': 'TRẠNG THÁI',
    'tinh_trang': 'TÌNH TRẠNG',

    // Numbering
    'stt': 'STT',
    'tt': 'TT',

    // Other common columns
    'ghi_chu': 'GHI CHÚ',
    'ma_hieu': 'MÃ HIỆU',
    'quy_cach': 'QUY CÁCH',
};

/**
 * Convert ASCII system key to Vietnamese display label
 * Fallback: Convert underscores to spaces and uppercase
 */
export function getDisplayLabel(key: string): string {
    return COLUMN_DISPLAY_MAP[key] || key.replace(/_/g, ' ').toUpperCase();
}

/**
 * Format number as Vietnamese currency
 */
export function formatVND(value: number): string {
    return new Intl.NumberFormat('vi-VN', {
        style: 'currency',
        currency: 'VND',
        maximumFractionDigits: 0
    }).format(value);
}

/**
 * Format number with thousand separators
 */
export function formatNumber(value: number): string {
    return new Intl.NumberFormat('vi-VN').format(value);
}
