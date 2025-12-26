export interface LineItem {
    id: string;
    stt: number;
    code: string;
    name: string;
    unit: string;
    quantity: number;
    price: number;
    total: number;
    status: 'valid' | 'warning' | 'error';
    notes: string;
}

const REAL_MATERIALS = [
    { name: "Thép Hòa Phát D10", code: "THEP-HP-D10", unit: "kg", price: 15800 },
    { name: "Xi măng PCB40", code: "XM-PCB40", unit: "bao", price: 82000 },
    { name: "Cát vàng sông Lô", code: "CAT-VANG", unit: "m3", price: 350000 },
    { name: "Gạch ống Tuynel", code: "GACH-ONG", unit: "viên", price: 1150 },
    { name: "Sơn Dulux ngoại thất", code: "SON-DULUX", unit: "lít", price: 185000 },
    { name: "Đá 1x2 Phủ Lý", code: "DA-1x2", unit: "m3", price: 290000 },
    { name: "Bê tông tươi M250", code: "BETONG-M250", unit: "m3", price: 1250000 },
];

const NOTES = ["Hàng về chậm", "Đã kiểm định", "Thiếu CO/CQ", "", "Đúng mẫu mã", "Chờ nghiệm thu"];

export function generateData(count: number = 1000): LineItem[] {
    const rows: LineItem[] = [];

    for (let i = 1; i <= count; i++) {
        const matIndex = i % REAL_MATERIALS.length;
        const material = REAL_MATERIALS[matIndex];
        const quantity = parseFloat((Math.random() * 500 + 10).toFixed(2));
        const unitPrice = material.price;
        const total = quantity * unitPrice;

        // Weighted status
        const rand = Math.random();
        const status = rand < 0.75 ? 'valid' : rand < 0.9 ? 'warning' : 'error';

        // Cycle through notes
        const note = NOTES[Math.floor(Math.random() * NOTES.length)];

        rows.push({
            id: `row_${i}`,
            stt: i,
            code: material.code,
            name: material.name,
            unit: material.unit,
            quantity: quantity,
            price: unitPrice,
            total: Math.round(total),
            status: status,
            notes: note,
        });
    }

    return rows;
}
