# UI LATENCY CONSTITUTION - TachFileTo

## ĐIỀU 1: TỐC ĐỘ TỐI THƯỢNG
- Mọi tương tác click → phản hồi trong ≤100ms
- Data grid: Virtual scrolling bắt buộc, 1M dòng vẫn 60fps
- File processing: UI KHÔNG bị trắng, phải hiển thị data cũ + badge "PROCESSING"

## ĐIỀU 2: CẤM HIỆU ỨNG TRANG TRÍ
- Cấm: transition, animation (trừ alert nguy hiểm cấp Red)
- Cấm: skeleton loading, count-up effect
- Cấm: typewriter effect, fade-in

## ĐIỀU 3: NGÔN NGỮ COCKPIT
- Sử dụng từ viết tắt chuẩn QS (ĐG, KL, T.TIỀN)
- Màu sắc: #DC2626 (Red) = Rủi ro, #059669 (Green) = An toàn
- Font: Monospace cho số liệu

## HÌNH PHẠT
- PR có animation = REJECT ngay tại review
- Component vi phạm = bị gỡ và viết lại
