// ui/src/lib/messages.vi.ts
// Single source of truth for ALL user-visible Vietnamese strings.
// RULE: No component may hardcode a user-facing string.
// Source of truth: PRODUCT_SPEC.md Section 10 (Language Policy)

import type { ProcessError } from './types';

// ─── Error Messages ────────────────────────────────────────────────────────────
export const ERROR_MESSAGES: Record<ProcessError, string> = {
    FileTooLarge: 'Tệp quá lớn. Vui lòng chọn tệp dưới 500MB.',
    OcrFailed: 'Không thể nhận dạng văn bản. Kiểm tra lại cài đặt OCR.',
    UnsupportedFormat: 'Định dạng tệp không được hỗ trợ. Chỉ chấp nhận PDF và DOCX.',
    UserCancelled: 'Đã hủy. Dữ liệu tạm thời đã được xóa.',
    IoError: 'Lỗi đọc tệp. Kiểm tra quyền truy cập thư mục.',
    EnginePanic: 'Lỗi hệ thống không xác định. Vui lòng thử lại.',
};

// ─── UI Strings ────────────────────────────────────────────────────────────────
export const UI = {
    // App identity
    app_name: 'TachFileTo',
    app_tagline: 'Xử lý tài liệu lớn. Offline hoàn toàn.',

    // Mode tabs
    mode_clean: 'Xử lý',
    mode_compare: 'So sánh',

    // Actions
    action_choose_file: 'Chọn tệp',
    action_export_md: 'Xuất Markdown',
    action_export_docx: 'Xuất DOCX',
    action_export_pdf: 'Xuất PDF',
    action_cancel: 'Hủy',
    action_copy_ai: 'Sao chép cho AI',
    action_compare: 'So sánh tài liệu',

    // Status phrases
    status_ready: 'Sẵn sàng',
    status_processing: 'Đang xử lý…',
    status_done: 'Hoàn tất',
    status_failed: 'Thất bại',
    status_exporting: 'Đang xuất file…',
    status_comparing: 'Đang so sánh…',

    // Drop zone
    dropzone_title: 'Kéo tệp vào đây',
    dropzone_subtitle: 'Hỗ trợ PDF và DOCX · Tối đa 500MB',
    dropzone_or: 'hoặc',

    // Progress
    progress_ocr: 'Đang nhận dạng văn bản (OCR)',
    progress_page: 'trang',
    progress_of: 'của',

    // Results
    result_title_clean: 'Kết quả xử lý',
    result_pages: 'Số trang',
    result_ocr_applied: 'Đã dùng OCR',
    result_no_ocr: 'Không cần OCR',
    result_markdown: 'Nội dung Markdown',

    // Compare
    compare_file_a: 'Tài liệu A (Phiên bản cũ)',
    compare_file_b: 'Tài liệu B (Phiên bản mới)',
    compare_identical: 'Hai tài liệu hoàn toàn giống nhau.',
    compare_changes: 'sai lệch được phát hiện',
    compare_added: 'Thêm mới',
    compare_removed: 'Đã xóa',
    compare_modified: 'Thay đổi',
    compare_location: 'Vị trí',
    compare_old: 'Giá trị cũ',
    compare_new: 'Giá trị mới',

    // Sidebar
    sidebar_title: 'Tệp trong phiên',
    sidebar_empty: 'Chưa có tệp nào',

    // Status bar
    statusbar_offline: 'Offline · Không có kết nối mạng',
    statusbar_version: 'v1.0.0',

    // Toast / notifications
    toast_copied: 'Đã sao chép vào clipboard',
    toast_exported: 'Đã xuất file thành công',
} as const;
