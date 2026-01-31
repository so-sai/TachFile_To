import sys
import os

# Đường dẫn tới thư mục chứa elite_pdf.pyd
sys.path.append(r"e:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\target\release")

try:
    import elite_pdf
    print("SUCCESS: Imported elite_pdf (Native)")
    
    # Tìm đại một file PDF trong thư mục dự án để test
    test_pdf = r"e:\DEV\elite_9_VN-ecosystem\app-tool-TachFileTo\test\pdf\BoQ 16052022-REV 3.pdf"
    
    # Thử gọi hàm
    try:
        doc = elite_pdf.EliteDocument(test_pdf)
        print(f"Page Count: {doc.count_pages()}")
    except Exception as e:
        print(f"EXPECTED ERROR (Opening MD as PDF): {e}")

except Exception as e:
    print(f"FAILED: {e}")
