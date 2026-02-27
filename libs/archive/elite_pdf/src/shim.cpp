#include <mupdf/fitz.h>
#include <mupdf/pdf.h>

// Dummy function to ensure this object file is linked and C++ runtime is pulled in
extern "C" void elite_shim_init() {
    fz_context *ctx = fz_new_context(NULL, NULL, FZ_STORE_UNLIMITED);
    if (ctx) {
        fz_drop_context(ctx);
    }
}
